<script setup lang="ts">
import { ref } from "vue";

const props = defineProps<{ server: string; port: number }>();
const emit = defineEmits<{ (e: "close"): void }>();

const copied = ref(false);
let copyTimer: number | undefined;

const address = () => `${props.server}:${props.port}`;

async function copyAddress() {
  const text = address();
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
    } else {
      // 兜底：临时 textarea 复制
      const ta = document.createElement("textarea");
      ta.value = text;
      ta.style.position = "fixed";
      ta.style.opacity = "0";
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      ta.remove();
    }
    copied.value = true;
    if (copyTimer) clearTimeout(copyTimer);
    copyTimer = window.setTimeout(() => (copied.value = false), 2000);
  } catch {
    // 复制失败：提示用户手动选择
    copied.value = false;
  }
}
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal" role="alertdialog" aria-modal="true" aria-labelledby="ks-title" aria-describedby="ks-desc">
      <svg class="m-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
        <polyline points="22 4 12 14.01 9 11.01" />
      </svg>

      <h3 id="ks-title" class="m-title">隧道已建立</h3>
      <p id="ks-desc" class="m-desc">在 Minecraft 中直接连接以下地址即可联机</p>

      <div class="addr-box">
        <span class="addr-text">{{ server }}:{{ port }}</span>
        <button class="copy-btn" type="button" :class="{ done: copied }" @click="copyAddress">
          <svg v-if="!copied" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <polyline points="20 6 9 17 4 12" />
          </svg>
          {{ copied ? "已复制" : "一键复制" }}
        </button>
      </div>

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
  color: #4ade80;
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
}

/* 地址 + 一键复制 */
.addr-box {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 10px 10px 14px;
  border-radius: 10px;
  background: rgba(74, 222, 128, 0.08);
  border: 1px solid rgba(74, 222, 128, 0.25);
}

.addr-text {
  flex: 1;
  min-width: 0;
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.01em;
  color: #f1e8e8;
  font-variant-numeric: tabular-nums;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.copy-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 7px 12px;
  border: none;
  border-radius: 8px;
  background: #4ade80;
  color: #0f1a12;
  font-family: inherit;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: filter 0.15s ease;
}

.copy-btn svg {
  width: 13px;
  height: 13px;
}

.copy-btn:hover {
  filter: brightness(1.1);
}

.copy-btn:active {
  filter: brightness(0.92);
}

.copy-btn.done {
  background: #22c55e;
}

.m-btn {
  width: 100%;
  margin-top: 4px;
  padding: 11px 14px;
  border: none;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(241, 232, 232, 0.8);
  font-family: inherit;
  font-size: 13.5px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease;
}

.m-btn:hover {
  background: rgba(255, 255, 255, 0.16);
  color: #f1e8e8;
}

.m-btn:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px rgba(74, 222, 128, 0.3);
}
</style>
