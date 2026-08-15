use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use futures_util::StreamExt;
use tauri::{Emitter, Manager};

/// 内核进程句柄（单实例：同一时刻只允许一个内核进程）
static KERNEL_CHILD: Mutex<Option<Child>> = Mutex::new(None);

/// 内核存放目录：<外壳 exe 所在目录>/kernel
/// 用 std::env::current_exe() 而非 app.path().executable_dir()，
/// 后者在部分环境下解析失败（回退到工作目录导致内核位置错乱）。
fn kernel_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("kernel")
}

/// 内核可执行文件路径
fn kernel_exe() -> PathBuf {
    kernel_dir().join("hongshi.exe")
}

/// 隧道状态文件路径
fn status_file() -> PathBuf {
    kernel_dir().join("tunnel.ini")
}

/// 自检：内核是否已存在于指定相对路径
#[tauri::command]
fn check_kernel() -> bool {
    kernel_exe().is_file()
}

/// 下载内核：请求下载接口拿签名 URL（主站失败回退镜像），下载到 kernel/hongshi.exe。
/// 通过 "kernel-download-progress" 事件推送 0.0–1.0 进度。
#[tauri::command]
async fn download_kernel(app: tauri::AppHandle) -> Result<String, String> {
    // 1. 获取签名下载地址（主站 → 镜像）
    let api_path = if cfg!(target_os = "windows") {
        "/api/download/windows".to_string()
    } else {
        let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "amd64" };
        let platform = if cfg!(target_os = "macos") { "darwin" } else { "linux" };
        format!("/api/download/{platform}?arch={arch}")
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("HTTP 客户端初始化失败：{e}"))?;

    let mut last_err = String::new();
    let mut download_url = None;
    for base in ["https://hongshi.site", "https://shithub.site"] {
        let url = format!("{base}{api_path}");
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if let Some(u) = json["url"].as_str() {
                            download_url = Some(u.to_string());
                            break;
                        }
                        last_err = "响应缺少 url 字段".into();
                    }
                    Err(e) => last_err = format!("响应解析失败：{e}"),
                }
            }
            Ok(resp) => {
                last_err = format!("HTTP {}", resp.status());
                if resp.status() == 429 {
                    last_err = "触发限流（60 秒内仅 1 次，每日 5 次），请稍后再试".into();
                }
            }
            Err(e) => last_err = e.to_string(),
        }
    }
    let url = download_url.ok_or_else(|| format!("获取下载地址失败（{last_err}）"))?;

    // 2. 流式下载到临时文件，原子改名
    let dir = kernel_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("创建内核目录失败：{e}"))?;
    let tmp = dir.join("hongshi.exe.download");
    let target = kernel_exe();

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("下载失败：{e}"))?;
    if !resp.status().is_success() {
        return Err(format!("下载失败：HTTP {}", resp.status()));
    }

    let total = resp.content_length().unwrap_or(0);
    let mut file = fs::File::create(&tmp).map_err(|e| format!("写入文件失败：{e}"))?;
    let mut stream = resp.bytes_stream();
    let mut done: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载中断：{e}"))?;
        file.write_all(&chunk).map_err(|e| format!("写入文件失败：{e}"))?;
        done += chunk.len() as u64;
        if total > 0 {
            let pct = (done as f64 / total as f64).clamp(0.0, 1.0);
            let _ = app.emit("kernel-download-progress", pct);
        }
    }
    file.flush().map_err(|e| format!("写入文件失败：{e}"))?;
    fs::rename(&tmp, &target).map_err(|e| format!("保存内核失败：{e}"))?;
    let _ = app.emit("kernel-download-progress", 1.0);
    Ok("内核下载完成".into())
}

/// 启动内核：hongshi.exe -server <中转服务器> -port <本地MC端口> -status-file <路径>
/// 单实例保护；退出时通过 "kernel-exited" 事件推送退出码。
#[tauri::command]
async fn start_kernel(
    app: tauri::AppHandle,
    server: String,
    port: u16,
) -> Result<(), String> {
    // 单实例检查
    let mut guard = KERNEL_CHILD.lock().unwrap();
    if let Some(child) = guard.as_mut() {
        match child.try_wait() {
            Ok(Some(_)) => *guard = None, // 已退出，允许重新拉起
            Ok(None) => return Err("内核已在运行".into()),
            Err(_) => *guard = None,
        }
    }

    let exe = kernel_exe();
    if !exe.is_file() {
        return Err("未找到内核程序，请先下载".into());
    }
    if server.trim().is_empty() {
        return Err("未选择服务器节点".into());
    }
    if port == 0 {
        return Err("端口号无效".into());
    }

    let dir = kernel_dir();
    let status = status_file();
    // stdout 丢弃（内核日志已写文件）；stderr 保留到 kernel/stderr.log——
    // panic/fatal 崩溃栈只走 stderr，之前丢弃导致崩溃原因完全不可见。
    let err_log = dir.join("stderr.log");
    let err_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&err_log)
        .map_err(|e| format!("打开 stderr 日志失败：{e}"))?;
    let child = Command::new(&exe)
        .current_dir(&dir)
        .args([
            "-server",
            server.trim(),
            "-port",
            &port.to_string(),
            "-status-file",
        ])
        .arg(&status)
        .stdout(Stdio::null())
        .stderr(err_file)
        .spawn()
        .map_err(|e| format!("启动内核失败：{e}"))?;

    *guard = Some(child);
    drop(guard);

    // 后台监控内核退出：巡检式 try_wait，不拿走 child（kernel_status 需要它报告
    // 运行状态），也不持锁等待（否则同步 command 主线程 lock 会堵死 UI）。
    let app2 = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(500));
        let exited: Option<String> = {
            let mut g = KERNEL_CHILD.lock().unwrap();
            match g.as_mut() {
                Some(child) => match child.try_wait() {
                    Ok(Some(status)) => {
                        let code = status
                            .code()
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "未知".to_string());
                        *g = None;
                        Some(code)
                    }
                    Ok(None) => None,
                    Err(_) => {
                        *g = None;
                        Some("未知".to_string())
                    }
                },
                None => None,
            }
        };
        if let Some(code) = exited {
            let _ = app2.emit("kernel-exited", code);
            break;
        }
    });
    Ok(())
}

/// 停止内核（kill 进程）
#[tauri::command]
fn stop_kernel() -> Result<(), String> {
    let mut guard = KERNEL_CHILD.lock().unwrap();
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    Ok(())
}

/// 内核运行状态：进程存活 + tunnel.ini 解析
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct KernelStatus {
    running: bool,
    status: String, // open | closed | unknown
    server: String,
    port: i32,
}

#[tauri::command]
fn kernel_status() -> KernelStatus {
    let running = {
        let mut guard = KERNEL_CHILD.lock().unwrap();
        match guard.as_mut() {
            Some(child) => child.try_wait().map(|w| w.is_none()).unwrap_or(false),
            None => false,
        }
    };

    let mut st = KernelStatus {
        running,
        status: "unknown".into(),
        server: String::new(),
        port: -1,
    };
    if let Ok(content) = fs::read_to_string(status_file()) {
        for line in content.lines() {
            if let Some(v) = line.strip_prefix("status=") {
                st.status = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("server=") {
                st.server = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("port=") {
                st.port = v.trim().parse().unwrap_or(-1);
            }
        }
    }
    st
}

/// 读取内核日志尾部（logs/<最新日期>.log 最后 40 行）
#[tauri::command]
fn kernel_log() -> String {
    match latest_log_path() {
        Some(path) => match fs::read_to_string(&path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let tail = &lines[lines.len().saturating_sub(40)..];
                tail.join("\n")
            }
            Err(_) => "(读取日志失败)".into(),
        },
        None => "(暂无日志)".into(),
    }
}

/// 最新的内核日志文件路径
fn latest_log_path() -> Option<PathBuf> {
    let logs_dir = kernel_dir().join("logs");
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    if let Ok(entries) = fs::read_dir(&logs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|x| x == "log").unwrap_or(false) {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(t) = meta.modified() {
                        if newest.as_ref().map(|(nt, _)| t > *nt).unwrap_or(true) {
                            newest = Some((t, path));
                        }
                    }
                }
            }
        }
    }
    newest.map(|(_, p)| p)
}

/// 用系统记事本打开内核日志（Windows 用 notepad.exe，其他平台用系统关联程序）
#[tauri::command]
fn open_kernel_log() -> Result<String, String> {
    let path = latest_log_path().ok_or("暂无日志文件")?;
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("notepad.exe")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开日志失败：{e}"))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = tauri_plugin_opener::open_path(
            path.to_str().unwrap_or(""),
            None::<String>,
        );
    }
    Ok(path.display().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            check_kernel,
            download_kernel,
            start_kernel,
            stop_kernel,
            kernel_status,
            kernel_log,
            open_kernel_log
        ])
        .setup(|app| {
            // 低分辨率屏幕自适应：
            // 窗口高度 = 屏幕可用高度（扣除任务栏约 48px，上限 720），宽度保持 9:16（上限 405）。
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(Some(monitor)) = window.current_monitor() {
                    let scale = monitor.scale_factor();
                    let logical_h = monitor.size().height as f64 / scale;
                    let avail_h = (logical_h - 48.0).max(480.0);
                    let target_h = avail_h.min(720.0);
                    let target_w = (target_h * 9.0 / 16.0).min(405.0);
                    if target_w >= 270.0 && target_h < 720.0 {
                        let _ = window.set_size(tauri::LogicalSize::new(
                            target_w.round(),
                            target_h.round(),
                        ));
                        let _ = window.center();
                    }
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
