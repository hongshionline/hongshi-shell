<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

// kind: "error" = 启动失败（红色警告）；"info" = 隧道正常关闭（中性提示）
defineProps<{ message: string; kind?: "error" | "info" }>();
const emit = defineEmits<{ (e: "close"): void }>();

const logHint = ref("");
const logError = ref("");

// 用系统记事本打开内核日志
async function openLog() {
  logError.value = "";
  try {
    const path = await invoke<string>("open_kernel_log");
    logHint.value = `已在记事本中打开日志\n${path}`;
  } catch (e) {
    logError.value = String(e);
  }
}
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal" role="alertdialog" aria-modal="true" aria-labelledby="kf-title" aria-describedby="kf-desc">
      <svg v-if="kind === 'info'" class="m-icon info" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="16" x2="12" y2="12" />
        <line x1="12" y1="8" x2="12.01" y2="8" />
      </svg>
      <svg v-else class="m-icon warn" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>

      <h3 id="kf-title" class="m-title">{{ kind === "info" ? "隧道已关闭" : "内核启动失败" }}</h3>
      <p id="kf-desc" class="m-desc">{{ message }}</p>

      <button class="log-btn" type="button" @click="openLog">
        查看内核日志（记事本）
      </button>
      <p v-if="logHint" class="log-hint">{{ logHint }}</p>
      <p v-else-if="logError" class="log-hint err">{{ logError }}</p>

      <button class="m-btn" type="button" @click="emit('close')">知道了</button>
    </div>
  </div>
</template>

<style scoped>
/* 全屏遮罩：z-index 低于底部导航（切换板块不受影响）与标题栏（可拖拽） */
.modal-mask {
  position: absolute;
  inset: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(8, 6, 7, 0.62);
  backdrop-filter: blur(3px);
  -webkit-backdrop-filter: blur(3px);
}

.modal {
  width: min(320px, 100%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 26px 22px 22px;
  border-radius: 16px;
  background: #251b1e;
  border: 1px solid rgba(255, 255, 255, 0.09);
  text-align: center;
}

.m-icon {
  width: 30px;
  height: 30px;
}

.m-icon.warn {
  color: #f87171;
}

.m-icon.info {
  color: #fbbf24;
}

.m-title {
  margin: 2px 0 0;
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: #f1e8e8;
}

.m-desc {
  margin: 0;
  font-size: 12.5px;
  line-height: 1.6;
  color: rgba(241, 232, 232, 0.55);
  word-break: break-all;
}

.log-btn {
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.05);
  padding: 8px 14px;
  color: rgba(241, 232, 232, 0.8);
  font-family: inherit;
  font-size: 12.5px;
  cursor: pointer;
  transition: background 0.15s ease, border-color 0.15s ease;
}

.log-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.25);
}

.log-hint {
  margin: 0;
  font-size: 11px;
  line-height: 1.6;
  color: rgba(241, 232, 232, 0.45);
  word-break: break-all;
  white-space: pre-line;
}

.log-hint.err {
  color: #fca5a5;
}

.m-btn {
  width: 100%;
  margin-top: 4px;
  padding: 11px 14px;
  border: none;
  border-radius: 10px;
  background: #ef4444;
  color: #fff;
  font-family: inherit;
  font-size: 13.5px;
  font-weight: 600;
  cursor: pointer;
  transition: filter 0.15s ease;
}

.m-btn:hover {
  filter: brightness(1.15);
}

.m-btn:active {
  filter: brightness(0.92);
}

.m-btn:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px rgba(248, 113, 113, 0.35);
}
</style>
