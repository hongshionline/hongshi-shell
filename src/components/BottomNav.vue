<script setup lang="ts">
export type TabId = "home" | "lobby" | "friends" | "profile";

interface TabItem {
  id: TabId;
  label: string;
  vb: string; // viewBox
  icon: string; // 实心 path
}

const props = defineProps<{ active: TabId }>();
const emit = defineEmits<{ (e: "select", id: TabId): void }>();

// 实心图标（fill currentColor）：激活红 / 默认灰
const tabs: TabItem[] = [
  {
    id: "home",
    label: "主页",
    vb: "0 0 24 24",
    icon: '<path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/>',
  },
  {
    id: "lobby",
    label: "联机大厅",
    vb: "0 0 1024 1024",
    icon: '<path d="M941 469.9c-43.9-197.4-105.9-246.1-143.6-265.6C737 173 666.9 191.2 643.7 203c-26.4 13.3-44.2 38.7-71 49.7-31 12.7-73.2 16.1-104.9 0.8-25.3-12.2-47.6-38.3-78.2-49.9-38.8-14.7-103.5-35.2-167.4 9.7C207 224 99.1 312.6 66.8 589.6c-20 170.8 75.5 225.7 72 223.2 24.6 17.3 79.6 46.4 154.2 0.7 49.8-30.5 90.3-90.9 90.3-90.9s18-33.6 42-47.8c16.9-10.1 42.3-2.4 42.3-2.4h89.7s28.5-3.5 48.4 5.1c19.4 8.3 30.4 28.8 30.4 28.8s59.1 74.9 115.7 113c24.1 16.2 121 34.9 171.1-25 32.5-39.1 54.8-159.6 18.1-324.4z m-479-17.5c0 15-12.2 27.1-27.2 27.1h-67.9v68c0 15-12.2 27.2-27.2 27.2h-20.4c-15 0-27.2-12.1-27.2-27.2v-68h-67.9c-15 0-27.2-12.1-27.2-27.1V432c0-15 12.2-27.2 27.2-27.2h67.9v-67.9c0-15 12.2-27.2 27.2-27.2h20.4c15 0 27.2 12.2 27.2 27.2v67.9h67.9c15 0 27.2 12.1 27.2 27.2v20.4z m191.4 95.1c-28.1 0-50.9-22.8-50.9-51 0-28.1 22.8-50.9 50.9-50.9 28.1 0 50.9 22.8 50.9 50.9 0 28.2-22.8 51-50.9 51z m93.4-113.2c-27.8 0-50.4-22.6-50.4-50.4s22.6-50.3 50.4-50.3c27.8 0 50.4 22.6 50.4 50.3-0.1 27.8-22.6 50.4-50.4 50.4z"/>',
  },
  {
    id: "friends",
    label: "好友",
    vb: "0 0 24 24",
    icon: '<path d="M16 11c1.66 0 2.99-1.34 2.99-3S17.66 5 16 5c-1.66 0-3 1.34-3 3s1.34 3 3 3zm-8 0c1.66 0 2.99-1.34 2.99-3S9.66 5 8 5C6.34 5 5 6.34 5 8s1.34 3 3 3zm0 2c-2.33 0-7 1.17-7 3.5V19h14v-2.5c0-2.33-4.67-3.5-7-3.5zm8 0c-.29 0-.62.02-.97.05 1.16.84 1.97 1.97 1.97 3.45V19h6v-2.5c0-2.33-4.67-3.5-7-3.5z"/>',
  },
  {
    id: "profile",
    label: "个人",
    vb: "0 0 24 24",
    icon: '<path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>',
  },
];
</script>

<template>
  <nav class="bottom-nav" aria-label="主导航">
    <button
      v-for="t in tabs"
      :key="t.id"
      class="nav-item"
      :class="{ active: t.id === props.active }"
      type="button"
      :aria-current="t.id === props.active ? 'page' : undefined"
      @click="emit('select', t.id)"
    >
      <span class="icon-wrap">
        <svg
          :viewBox="t.vb"
          fill="currentColor"
          aria-hidden="true"
          v-html="t.icon"
        />
      </span>
      <span class="label">{{ t.label }}</span>
    </button>
  </nav>
</template>

<style scoped>
/* 浮层式毛玻璃导航栏：内容从下方滚动穿过；z-index 高于全局弹窗遮罩 */
.bottom-nav {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 20;
  display: flex;
  height: calc(62px + env(safe-area-inset-bottom));
  padding-bottom: env(safe-area-inset-bottom);
  background: rgba(17, 13, 14, 0.5);
  backdrop-filter: blur(18px) saturate(1.6) brightness(1.03);
  -webkit-backdrop-filter: blur(18px) saturate(1.6) brightness(1.03);
  border-top: 1px solid rgba(255, 255, 255, 0.07);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.09);
  user-select: none;
  -webkit-user-select: none;
}

.nav-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  border: none;
  background: transparent;
  padding: 0;
  cursor: pointer;
  color: rgba(241, 232, 232, 0.45);
  font-family: inherit;
  transition: color 0.18s ease;
}

.nav-item:hover {
  color: rgba(241, 232, 232, 0.8);
}

.nav-item:focus-visible {
  outline: 2px solid rgba(248, 113, 113, 0.8);
  outline-offset: -2px;
  border-radius: 12px;
}

.icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 28px;
  transition: transform 0.3s ease-out;
}

.nav-item:not(.active):hover .icon-wrap {
  transform: scale(1.15);
}

.nav-item svg {
  width: 21px;
  height: 21px;
}

.label {
  font-size: 10.5px;
  line-height: 1;
  letter-spacing: 0.02em;
}

/* 激活态：红色实心图标 + 弹簧放大（红石能量） */
.nav-item.active {
  color: #f87171;
}

.nav-item.active .icon-wrap {
  transform: scale(1.06);
  animation: icon-pop 0.45s ease-out;
}

@keyframes icon-pop {
  0% {
    transform: scale(0.8);
  }
  55% {
    transform: scale(1.14);
  }
  78% {
    transform: scale(0.97);
  }
  100% {
    transform: scale(1.06);
  }
}
</style>
