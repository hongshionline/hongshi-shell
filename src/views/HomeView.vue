<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { fetch } from "@tauri-apps/plugin-http";
import { invoke } from "@tauri-apps/api/core";
import type { KernelState } from "../components/KernelDownloadModal.vue";
import NoticeCarousel from "../components/NoticeCarousel.vue";

const PRIMARY_URL = "https://hongshi.site/newserver.json";
const FALLBACK_URL = "https://shithub.site/newserver.json";
const REQUEST_TIMEOUT_MS = 6000;

// 内核状态由 App 全局管理（弹窗展示），这里只用于控制启动按钮可用性
const props = defineProps<{ kernelState: KernelState }>();
const emit = defineEmits<{
  (e: "kernel-start"): void;
  (e: "kernel-stop-request"): void;
  (e: "kernel-fail", message: string): void;
  (e: "kernel-open", tunnel: { server: string; port: number }): void;
}>();

// ---------- 服务器列表 ----------
const servers = ref<Record<string, string>>({});
const loading = ref(false);
const error = ref("");
const selected = ref("");
const port = ref(25565);

const serverEntries = computed(() => Object.entries(servers.value));

const canStart = computed(() => {
  const p = Number(port.value);
  return (
    props.kernelState === "ready" &&
    !loading.value &&
    selected.value !== "" &&
    Number.isInteger(p) &&
    p >= 1 &&
    p <= 65535
  );
});

async function loadServers() {
  loading.value = true;
  error.value = "";
  let lastErr = "";
  for (const url of [PRIMARY_URL, FALLBACK_URL]) {
    try {
      const resp = await fetch(url, { signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS) });
      if (!resp.ok) {
        lastErr = `HTTP ${resp.status}`;
        continue;
      }
      const data = await resp.json();
      if (data && typeof data === "object" && Object.keys(data).length > 0) {
        servers.value = data as Record<string, string>;
        const first = Object.values(data as Record<string, string>)[0];
        if (first) selected.value = first;
        loading.value = false;
        return;
      }
      lastErr = "返回内容为空";
    } catch (e) {
      lastErr = String(e);
    }
  }
  loading.value = false;
  error.value = `无法获取服务器列表（${lastErr}）`;
  servers.value = {};
}

// ---------- 内核启动 / 状态 ----------
type RunState = "idle" | "starting" | "connecting" | "open" | "failed" | "stopped";

interface KernelStatus {
  running: boolean;
  status: string;
  server: string;
  port: number;
}

const runState = ref<RunState>("idle");
const runError = ref("");
const tunnel = ref({ server: "", port: -1 });
const stopRequested = ref(false);

let pollTimer: number | undefined;

// 用系统记事本打开内核日志
async function openLog() {
  try {
    await invoke("open_kernel_log");
  } catch (e) {
    runError.value = `打开日志失败：${e}`;
  }
}

function stopPolling() {
  if (pollTimer !== undefined) {
    clearInterval(pollTimer);
    pollTimer = undefined;
  }
}

function startPolling() {
  stopPolling();
  pollTimer = window.setInterval(async () => {
    try {
      const st = await invoke<KernelStatus>("kernel_status");
      if (!st.running) {
        stopPolling();
        if (stopRequested.value) {
          runState.value = "stopped";
        } else {
          runState.value = "failed";
          emit("kernel-fail", "内核进程已退出");
        }
        return;
      }
      if (st.status === "open" && st.port > 0) {
        const wasOpen = runState.value === "open";
        tunnel.value = { server: st.server || selected.value, port: st.port };
        runState.value = "open";
        // 状态从非 open 变为 open 时，通知 App 弹成功弹窗（仅一次）
        if (!wasOpen) {
          emit("kernel-open", tunnel.value);
        }
      } else {
        runState.value = "connecting";
      }
    } catch {
      // 单次查询失败忽略，下轮重试
    }
  }, 500);
}

async function onStart() {
  emit("kernel-start");
  runState.value = "starting";
  runError.value = "";
  stopRequested.value = false;
  try {
    await invoke("start_kernel", { server: selected.value, port: Number(port.value) });
    runState.value = "connecting";
    startPolling();
  } catch (e) {
    runError.value = String(e);
    runState.value = "failed";
    emit("kernel-fail", String(e));
  }
}

async function onStop() {
  stopRequested.value = true;
  emit("kernel-stop-request");
  try {
    await invoke("stop_kernel");
  } catch {
    // 忽略
  }
}

onMounted(() => {
  loadServers();
});

onBeforeUnmount(() => {
  stopPolling();
});
</script>

<template>
  <section class="home">
    <!-- 顶部展示卡片（每日资讯等，骨架） -->
    <NoticeCarousel />

    <!-- 连接设置 -->
    <div class="card">
      <div class="card-head">
        <h2 class="card-title">连接设置</h2>
        <span v-if="loading" class="spinner" aria-hidden="true" />
      </div>

      <!-- 服务器节点 -->
      <label class="field-label" for="server-select">服务器节点</label>
      <div class="select-wrap">
        <select
          id="server-select"
          v-model="selected"
          class="select"
          :disabled="loading || runState === 'connecting' || runState === 'open'"
        >
          <option v-if="!loading && !error" value="" disabled>选择节点…</option>
          <option v-for="[name, host] in serverEntries" :key="host" :value="host">
            {{ name }} · {{ host }}
          </option>
        </select>
        <svg class="select-arrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </div>

      <p v-if="error" class="error" role="alert">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="8" x2="12" y2="12" />
          <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
        <span>{{ error }}</span>
        <button class="retry" type="button" @click="loadServers">重试</button>
      </p>
      <p v-else-if="!loading" class="field-hint">
        {{ Object.keys(servers).length ? `主站获取成功 · ${Object.keys(servers).length} 个节点` : "从主站获取节点，失败自动回退国内镜像" }}
      </p>

      <!-- 端口号 -->
      <label class="field-label" for="port-input">端口号</label>
      <input
        id="port-input"
        v-model.number="port"
        class="input"
        type="number"
        min="1"
        max="65535"
        placeholder="25565"
        inputmode="numeric"
        :disabled="runState === 'connecting' || runState === 'open'"
      />
      <p class="field-hint">本地监听端口，范围 1 – 65535</p>

      <!-- 启动 / 运行状态 -->
      <template v-if="runState === 'idle' || runState === 'stopped' || runState === 'failed'">
        <button class="primary" type="button" :disabled="!canStart" @click="onStart">
          <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path d="M8 5v14l11-7z" />
          </svg>
          {{ runState === "failed" ? "重新启动内核" : "启动内核" }}
        </button>
        <p v-if="runState === 'stopped'" class="state-line stopped">已停止联机</p>
      </template>

      <button v-else-if="runState === 'starting'" class="primary" type="button" disabled>
        <span class="spinner small" aria-hidden="true" />
        正在启动…
      </button>

      <div v-else-if="runState === 'connecting'" class="state-panel">
        <span class="spinner small" aria-hidden="true" />
        <span class="state-line">正在连接中转服务器…</span>
      </div>

      <div v-else-if="runState === 'open'" class="state-panel open">
        <div class="tunnel-card">
          <p class="tunnel-label">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
              <polyline points="22 4 12 14.01 9 11.01" />
            </svg>
            隧道已建立
          </p>
          <p class="tunnel-addr">{{ tunnel.server }}:{{ tunnel.port }}</p>
          <p class="tunnel-hint">在 Minecraft 中直接连接此地址即可联机</p>
          <button class="stop-btn" type="button" @click="onStop">停止联机</button>
        </div>
      </div>

      <!-- 启动失败：提示 + 记事本查看日志 -->
      <div v-if="runState === 'failed'" class="fail-panel" role="alert">
        <p class="fail-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
          启动失败：{{ runError }}
        </p>
        <button class="log-toggle" type="button" @click="openLog">查看内核日志（记事本）</button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.home {
  padding: 16px;
}

/* ---------- 连接设置卡片（扁平化） ---------- */
.card {
  padding: 6px 4px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.card-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: #f1e8e8;
}

/* 加载 spinner */
.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(248, 113, 113, 0.2);
  border-top-color: #f87171;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.spinner.small {
  width: 14px;
  height: 14px;
  border-width: 2px;
  flex-shrink: 0;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.field-label {
  font-size: 12px;
  font-weight: 500;
  color: rgba(241, 232, 232, 0.6);
  margin-top: 6px;
}

/* 下拉框 */
.select-wrap {
  position: relative;
}

.select {
  width: 100%;
  height: 42px;
  appearance: none;
  -webkit-appearance: none;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 10px;
  background: rgba(0, 0, 0, 0.28);
  color: #f1e8e8;
  font-family: inherit;
  font-size: 13.5px;
  padding: 0 38px 0 12px;
  cursor: pointer;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.select:hover:not(:disabled) {
  border-color: rgba(255, 255, 255, 0.22);
}

.select:focus-visible {
  outline: none;
  border-color: #f87171;
  box-shadow: 0 0 0 3px rgba(248, 113, 113, 0.18);
}

.select:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.select option {
  background: #1a1315;
  color: #f1e8e8;
}

.select-arrow {
  position: absolute;
  right: 12px;
  top: 50%;
  width: 15px;
  height: 15px;
  transform: translateY(-50%);
  color: rgba(241, 232, 232, 0.5);
  pointer-events: none;
}

/* 输入框 */
.input {
  width: 100%;
  height: 42px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 10px;
  background: rgba(0, 0, 0, 0.28);
  color: #f1e8e8;
  font-family: inherit;
  font-size: 13.5px;
  padding: 0 12px;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.input:hover:not(:disabled) {
  border-color: rgba(255, 255, 255, 0.22);
}

.input:focus-visible {
  outline: none;
  border-color: #f87171;
  box-shadow: 0 0 0 3px rgba(248, 113, 113, 0.18);
}

.input:disabled {
  opacity: 0.55;
}

.input::-webkit-outer-spin-button,
.input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.input::placeholder {
  color: rgba(241, 232, 232, 0.3);
}

.field-hint {
  margin: 2px 0 0;
  font-size: 11.5px;
  color: rgba(241, 232, 232, 0.4);
}

/* 错误状态（可恢复） */
.error {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 2px 0 0;
  font-size: 12px;
  line-height: 1.5;
  color: #fca5a5;
}

.error svg {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.retry {
  flex-shrink: 0;
  border: 1px solid rgba(248, 113, 113, 0.4);
  border-radius: 6px;
  background: rgba(248, 113, 113, 0.12);
  color: #fca5a5;
  font-size: 11.5px;
  font-family: inherit;
  padding: 2px 10px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.retry:hover {
  background: rgba(248, 113, 113, 0.22);
}

/* 主按钮（扁平化：纯色 + 无发光） */
.primary {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  height: 44px;
  margin-top: 10px;
  border: none;
  border-radius: 10px;
  background: #ef4444;
  color: #fff;
  font-family: inherit;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.02em;
  cursor: pointer;
  transition: filter 0.15s ease, transform 0.15s ease;
}

.primary svg {
  width: 16px;
  height: 16px;
}

.primary:hover:not(:disabled) {
  filter: brightness(1.15);
  transform: translateY(-1px);
}

.primary:active:not(:disabled) {
  transform: translateY(0);
  filter: brightness(0.92);
}

.primary:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px rgba(248, 113, 113, 0.35);
}

.primary:disabled {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(241, 232, 232, 0.35);
  cursor: not-allowed;
}

/* 运行状态 */
.state-panel {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-top: 12px;
}

.state-line {
  margin: 0;
  font-size: 12.5px;
  color: rgba(241, 232, 232, 0.55);
}

.state-line.stopped {
  text-align: center;
  margin-top: 8px;
}

/* 隧道建立卡片 */
.tunnel-card {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 14px;
  border-radius: 14px;
  background: rgba(74, 222, 128, 0.08);
  border: 1px solid rgba(74, 222, 128, 0.25);
}

.tunnel-label {
  display: flex;
  align-items: center;
  gap: 5px;
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  color: #4ade80;
}

.tunnel-label svg {
  width: 14px;
  height: 14px;
}

.tunnel-addr {
  margin: 2px 0 0;
  font-size: 17px;
  font-weight: 700;
  letter-spacing: 0.01em;
  color: #f1e8e8;
  font-variant-numeric: tabular-nums;
}

.tunnel-hint {
  margin: 0;
  font-size: 11px;
  color: rgba(241, 232, 232, 0.45);
}

.stop-btn {
  margin-top: 8px;
  padding: 6px 22px;
  border: 1px solid rgba(248, 113, 113, 0.45);
  border-radius: 8px;
  background: transparent;
  color: #f87171;
  font-family: inherit;
  font-size: 12.5px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.stop-btn:hover {
  background: rgba(248, 113, 113, 0.12);
}

/* 启动失败：提示 + 日志 */
.fail-panel {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
}

.fail-title {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0;
  font-size: 12.5px;
  line-height: 1.5;
  color: #fca5a5;
  text-align: center;
}

.fail-title svg {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.log-toggle {
  border: none;
  background: none;
  padding: 2px 8px;
  color: rgba(241, 232, 232, 0.55);
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
  border-radius: 6px;
  transition: background 0.15s ease, color 0.15s ease;
}

.log-toggle:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #f1e8e8;
}
</style>
