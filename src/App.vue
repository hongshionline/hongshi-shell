<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import BottomNav, { type TabId } from "./components/BottomNav.vue";
import KernelDownloadModal, { type KernelState } from "./components/KernelDownloadModal.vue";
import KernelFailModal from "./components/KernelFailModal.vue";
import KernelSuccessModal from "./components/KernelSuccessModal.vue";
import KernelUpdateModal from "./components/KernelUpdateModal.vue";
import ShellUpdateModal from "./components/ShellUpdateModal.vue";
import HomeView from "./views/HomeView.vue";
import LobbyView from "./views/LobbyView.vue";
import FriendsView from "./views/FriendsView.vue";
import ProfileView from "./views/ProfileView.vue";

const appWindow = getCurrentWindow();

const activeTab = ref<TabId>("home");

const titles: Record<TabId, string> = {
  home: "主页",
  lobby: "联机大厅",
  friends: "好友",
  profile: "个人",
};

// ---------- 内核自检（全局弹窗） ----------
const kernelState = ref<KernelState>("checking");
const kernelProgress = ref(0);
const kernelError = ref("");

// ---------- 内核更新检查 ----------
const kernelUpdate = ref({ show: false, current: "", latest: "" });

// ---------- 外壳更新 ----------
const shellUpdate = ref({ show: false, current: 0, latest: 0 });
const shellProgress = ref(0);

async function checkShellUpdate() {
  try {
    const info = await invoke<{ available: boolean; current: number; latest: number }>(
      "check_shell_update"
    );
    if (info.available) {
      shellUpdate.value = { show: true, current: info.current, latest: info.latest };
    }
  } catch {
    // 检查失败不打扰
  }
}

async function startShellDownload() {
  shellProgress.value = 0;
  try {
    await invoke("download_shell");
    // 更新脚本已启动：提示后关闭窗口（bat 负责替换并重启）
    setTimeout(() => {
      shellUpdate.value.show = false;
      appWindow.close();
    }, 1500);
  } catch (e) {
    shellUpdate.value.show = false;
    console.error("shell update failed:", e);
  }
}

async function checkKernel() {
  kernelState.value = "checking";
  try {
    const ok = await invoke<boolean>("check_kernel");
    kernelState.value = ok ? "ready" : "missing";
    if (ok) {
      // 内核就绪后检查是否有新版本
      try {
        const info = await invoke<{ available: boolean; current: string; latest: string }>(
          "check_kernel_update"
        );
        if (info.available) {
          kernelUpdate.value = { show: true, current: info.current, latest: info.latest };
        }
      } catch {
        // 更新检查失败不打扰（网络问题等）
      }
      checkShellUpdate();
    }
  } catch (e) {
    kernelError.value = String(e);
    kernelState.value = "error";
  }
}

async function startKernelDownload() {
  kernelState.value = "downloading";
  kernelProgress.value = 0;
  kernelError.value = "";
  try {
    await invoke("download_kernel");
    kernelState.value = "ready";
    kernelUpdate.value.show = false; // 更新完成，关闭更新弹窗
    checkKernel(); // 重新检查（更新后版本已刷新）
  } catch (e) {
    kernelError.value = String(e);
    kernelState.value = "error";
    kernelUpdate.value.show = false; // 关闭更新弹窗，让下载失败弹窗可见
  }
}

let unlistenProgress: UnlistenFn | undefined;
let unlistenExited: UnlistenFn | undefined;
let unlistenShellProgress: UnlistenFn | undefined;
onMounted(() => {
  checkKernel();
  listen<number>("kernel-download-progress", (e) => {
    kernelProgress.value = e.payload;
  }).then((un) => (unlistenProgress = un));

  listen<number>("shell-download-progress", (e) => {
    shellProgress.value = e.payload;
  }).then((un) => (unlistenShellProgress = un));

  // 内核退出全局监听（App 常驻，切换板块也不会错过）：
  // 主动停止（stop-request）不弹窗；退出码 0 = 正常关闭（隧道被回收/连接断开），
  // 弹中性提示；非 0 = 异常退出，弹失败提示
  listen<string>("kernel-exited", (e) => {
    if (kernelStopRequested.value) {
      kernelStopRequested.value = false;
      return;
    }
    const code = e.payload;
    if (code === "0") {
      openKernelInfo("隧道已关闭（连接中断或服务器回收）");
    } else if (code === "1") {
      openKernelFail("隧道创建失败（服务器不可达或拒绝连接）");
    } else if (code === "2") {
      openKernelFail("内核参数错误，请检查配置");
    } else {
      openKernelFail(`内核异常退出（退出码 ${code}）`);
    }
  }).then((un) => (unlistenExited = un));
});

onBeforeUnmount(() => {
  unlistenProgress?.();
  unlistenExited?.();
  unlistenShellProgress?.();
});

// ---------- 启动失败弹窗 ----------
const kernelFail = ref({ show: false, message: "" });
const kernelStopRequested = ref(false);

function openKernelFail(message: string) {
  kernelFail.value = { show: true, message };
}

// ---------- 隧道关闭中性提示 ----------
const kernelInfo = ref({ show: false, message: "" });

function openKernelInfo(message: string) {
  kernelInfo.value = { show: true, message };
}

// ---------- 隧道建立成功弹窗 ----------
const kernelOpen = ref({ show: false, server: "", port: 0 });

function onKernelOpen(t: { server: string; port: number }) {
  kernelOpen.value = { show: true, server: t.server, port: t.port };
}

function onKernelStart() {
  kernelStopRequested.value = false;
}

function onKernelStopRequest() {
  kernelStopRequested.value = true;
}

async function closeWindow() {
  try {
    await appWindow.close();
  } catch (err) {
    console.error("close failed:", err);
  }
}

// 无边框窗口拖拽：标题栏按下时显式调用 startDragging
function onTitlebarMousedown(e: MouseEvent) {
  if ((e.target as HTMLElement).closest("button")) return;
  appWindow.startDragging();
}
</script>

<template>
  <div class="panel">
    <header class="titlebar" @mousedown="onTitlebarMousedown">
      <span class="app-name">{{ titles[activeTab] }}</span>
      <button class="btn-close" title="关闭" @mousedown.stop @click="closeWindow">✕</button>
    </header>

    <main class="content">
      <Transition name="view" mode="out-in">
        <HomeView
          v-if="activeTab === 'home'"
          key="home"
          :kernel-state="kernelState"
          @kernel-start="onKernelStart"
          @kernel-stop-request="onKernelStopRequest"
          @kernel-fail="openKernelFail"
          @kernel-open="onKernelOpen"
        />
        <LobbyView v-else-if="activeTab === 'lobby'" key="lobby" />
        <FriendsView v-else-if="activeTab === 'friends'" key="friends" />
        <ProfileView v-else key="profile" />
      </Transition>
    </main>

    <BottomNav :active="activeTab" @select="activeTab = $event" />

    <!-- 内核未就绪全局弹窗：不可关闭，只能下载；不遮挡底部导航 -->
    <KernelDownloadModal
      v-if="kernelState === 'missing' || kernelState === 'downloading' || kernelState === 'error'"
      :state="kernelState"
      :progress="kernelProgress"
      :error="kernelError"
      @download="startKernelDownload"
    />

    <!-- 启动失败弹窗（可关闭，含日志查看） -->
    <KernelFailModal
      v-if="kernelFail.show"
      :message="kernelFail.message"
      kind="error"
      @close="kernelFail.show = false"
    />

    <!-- 隧道关闭提示（退出码 0，中性） -->
    <KernelFailModal
      v-if="kernelInfo.show"
      :message="kernelInfo.message"
      kind="info"
      @close="kernelInfo.show = false"
    />

    <!-- 隧道建立成功弹窗（一键复制联机地址） -->
    <KernelSuccessModal
      v-if="kernelOpen.show"
      :server="kernelOpen.server"
      :port="kernelOpen.port"
      @close="kernelOpen.show = false"
    />

    <!-- 内核新版本提示弹窗 -->
    <KernelUpdateModal
      v-if="kernelUpdate.show"
      :current="kernelUpdate.current"
      :latest="kernelUpdate.latest"
      :progress="kernelProgress"
      @close="kernelUpdate.show = false"
      @update="startKernelDownload"
    />

    <!-- 外壳新版本提示弹窗 -->
    <ShellUpdateModal
      v-if="shellUpdate.show"
      :current="shellUpdate.current"
      :latest="shellUpdate.latest"
      :progress="shellProgress"
      @close="shellUpdate.show = false"
      @update="startShellDownload"
    />
  </div>
</template>

<style>
/* 窗口不透明，圆角/阴影由系统（Win11 DWM / macOS）提供，Linux 保持直角 */
html,
body {
  margin: 0;
  padding: 0;
  height: 100%;
  background: #130e10;
  overflow: hidden;
}

#app {
  height: 100%;
}

* {
  box-sizing: border-box;
}

::selection {
  background: rgba(248, 113, 113, 0.35);
  color: #fdf2f2;
}

/* 视图占位（各 tab 内容区共享） */
.view {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 24px;
  text-align: center;
}

.view-icon {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 20px;
  background: rgba(248, 113, 113, 0.08);
  color: rgba(248, 113, 113, 0.8);
  margin-bottom: 6px;
}

.view-icon svg {
  width: 30px;
  height: 30px;
}

.view-title {
  margin: 0;
  font-size: 19px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: #f1e8e8;
}

.view-desc {
  margin: 0;
  max-width: 30ch;
  font-size: 13px;
  line-height: 1.6;
  color: rgba(241, 232, 232, 0.6);
}

/* 视图切换过渡（150–250ms，传达状态） */
.view-enter-active,
.view-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.view-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.view-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>

<style scoped>
.panel {
  position: relative;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(160deg, #1e1619 0%, #130e10 100%);
  color: #f1e8e8;
  font-family: "Segoe UI", "Microsoft YaHei", system-ui, sans-serif;
}

.titlebar {
  position: relative;
  z-index: 15; /* 高于内核弹窗遮罩(10)，保证弹窗状态下仍可拖拽/关闭 */
  height: 44px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 10px 0 18px;
  background: rgba(255, 255, 255, 0.04);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  user-select: none;
  -webkit-user-select: none;
}

.app-name {
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: rgba(241, 232, 232, 0.85);
}

.btn-close {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: rgba(241, 232, 232, 0.6);
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.btn-close:hover {
  background: rgba(232, 60, 60, 0.9);
  color: #fff;
}

.content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  /* 内容滚动时从底部玻璃导航下穿过 */
  padding-bottom: calc(62px + env(safe-area-inset-bottom));
}

/* 深色主题滚动条 */
.content::-webkit-scrollbar {
  width: 4px;
}

.content::-webkit-scrollbar-thumb {
  background: rgba(248, 113, 113, 0.25);
  border-radius: 2px;
}

.content::-webkit-scrollbar-thumb:hover {
  background: rgba(248, 113, 113, 0.4);
}

.content::-webkit-scrollbar-track {
  background: transparent;
}
</style>
