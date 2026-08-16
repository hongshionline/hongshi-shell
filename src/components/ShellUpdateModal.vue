<script setup lang="ts">
import { ref } from "vue";

defineProps<{ current: number; latest: number; progress: number }>();
const emit = defineEmits<{ (e: "close"): void; (e: "update"): void }>();

const phase = ref<"prompt" | "downloading" | "done">("prompt");
</script>

<template>
  <div class="modal-mask" @click.self="phase === 'prompt' && emit('close')">
    <div class="modal" role="alertdialog" aria-modal="true" aria-labelledby="su-title" aria-describedby="su-desc">
      <svg class="m-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
        <polyline points="7 10 12 15 17 10" />
        <line x1="12" y1="15" x2="12" y2="3" />
      </svg>

      <h3 id="su-title" class="m-title">
        {{ phase === "prompt" ? "发现外壳新版本" : phase === "downloading" ? "正在更新外壳…" : "更新完成" }}
      </h3>

      <div v-if="phase === 'prompt'" class="ver-row">
        <span class="ver-item">
          <span class="ver-label">当前版本</span>
          <span class="ver-value">v{{ current }}</span>
        </span>
        <svg class="ver-arrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <line x1="5" y1="12" x2="19" y2="12" />
          <polyline points="12 5 19 12 12 19" />
        </svg>
        <span class="ver-item new">
          <span class="ver-label">最新版本</span>
          <span class="ver-value">v{{ latest }}</span>
        </span>
      </div>

      <div v-else-if="phase === 'downloading'" class="progress" role="progressbar" :aria-valuenow="Math.round(progress * 100)" aria-valuemin="0" aria-valuemax="100">
        <div class="progress-bar" :style="{ transform: `scaleX(${progress})` }" />
      </div>

      <p class="m-desc" :id="phase === 'prompt' ? undefined : 'su-desc'">
        <template v-if="phase === 'prompt'">更新后外壳将自动重启，内核连接不受影响</template>
        <template v-else-if="phase === 'downloading'">正在下载新版本，请稍候。</template>
        <template v-else>外壳即将自动重启，请稍候…</template>
      </p>

      <div v-if="phase === 'prompt'" class="btn-row">
        <button class="m-btn ghost" type="button" @click="emit('close')">稍后</button>
        <button class="m-btn" type="button" @click="phase = 'downloading'; emit('update')">立即更新</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 全屏遮罩：z-index 低于底部导航与标题栏 */
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
  gap: 12px;
  padding: 26px 22px 22px;
  border-radius: 16px;
  background: #251b1e;
  border: 1px solid rgba(255, 255, 255, 0.09);
  text-align: center;
}

.m-icon {
  width: 30px;
  height: 30px;
  color: #f87171;
}

.m-title {
  margin: 2px 0 0;
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: #f1e8e8;
}

.ver-row {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
}

.ver-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 10px 0;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.05);
}

.ver-item.new {
  background: rgba(248, 113, 113, 0.1);
}

.ver-label {
  font-size: 10.5px;
  color: rgba(241, 232, 232, 0.45);
}

.ver-value {
  font-size: 16px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: #f1e8e8;
}

.ver-item.new .ver-value {
  color: #f87171;
}

.ver-arrow {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  color: rgba(241, 232, 232, 0.4);
}

.progress {
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.08);
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  border-radius: 2px;
  background: #ef4444;
  transform-origin: left;
  transition: transform 0.15s ease;
}

.m-desc {
  margin: 0;
  font-size: 12px;
  line-height: 1.6;
  color: rgba(241, 232, 232, 0.5);
}

.btn-row {
  display: flex;
  gap: 8px;
  width: 100%;
}

.m-btn {
  flex: 1;
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

.m-btn.ghost {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(241, 232, 232, 0.75);
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
