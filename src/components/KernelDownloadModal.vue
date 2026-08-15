<script setup lang="ts">
export type KernelState = "checking" | "ready" | "missing" | "downloading" | "error";

defineProps<{ state: KernelState; progress: number; error: string }>();
const emit = defineEmits<{ (e: "download"): void }>();
</script>

<template>
  <div class="modal-mask">
    <div class="modal" role="alertdialog" aria-modal="true" aria-labelledby="kd-title" aria-describedby="kd-desc">
      <svg
        v-if="state === 'missing' || state === 'error'"
        class="m-icon warn"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
      <svg
        v-else-if="state === 'downloading'"
        class="m-icon dl"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
        <polyline points="7 10 12 15 17 10" />
        <line x1="12" y1="15" x2="12" y2="3" />
      </svg>

      <h3 id="kd-title" class="m-title">
        <template v-if="state === 'missing'">未检测到“红石联机内核”</template>
        <template v-else-if="state === 'downloading'">正在下载内核… {{ Math.round(progress * 100) }}%</template>
        <template v-else>内核下载失败</template>
      </h3>

      <p id="kd-desc" class="m-desc">
        <template v-if="state === 'missing'">内核负责连接中转服务器并创建隧道，下载完成后即可开始联机。</template>
        <template v-else-if="state === 'downloading'">正在获取适合您设备的最新版本，请稍候。</template>
        <template v-else>{{ error }}</template>
      </p>

      <div v-if="state === 'downloading'" class="progress" role="progressbar" :aria-valuenow="Math.round(progress * 100)" aria-valuemin="0" aria-valuemax="100">
        <div class="progress-bar" :style="{ transform: `scaleX(${progress})` }" />
      </div>

      <button v-if="state === 'missing'" class="m-btn" type="button" @click="emit('download')">
        点击下载适合您的设备的最新版内核程序
      </button>
      <button v-else-if="state === 'error'" class="m-btn" type="button" @click="emit('download')">
        重新下载
      </button>
    </div>
  </div>
</template>

<style scoped>
/* 全屏遮罩：悬浮于主界面之上；z-index 低于底部导航（切换板块不受影响） */
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

.m-icon.dl {
  color: #f87171;
  animation: bob 1.2s ease-in-out infinite;
}

@keyframes bob {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(3px);
  }
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

.progress {
  width: 100%;
  height: 4px;
  margin: 6px 0 2px;
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

.m-btn {
  width: 100%;
  margin-top: 6px;
  padding: 11px 14px;
  border: none;
  border-radius: 10px;
  background: #ef4444;
  color: #fff;
  font-family: inherit;
  font-size: 13.5px;
  font-weight: 600;
  line-height: 1.5;
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
