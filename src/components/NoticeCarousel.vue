<script setup lang="ts">
import { ref } from "vue";

interface BannerItem {
  title: string;
  color: string;
}

// 骨架：纯色占位卡片，后续替换为每日资讯等展示内容
const items = ref<BannerItem[]>([
  { title: "每日资讯即将上线", color: "#ef4444" },
  { title: "联机大厅即将上线", color: "#b91c1c" },
  { title: "敬请期待", color: "#7f1d1d" },
]);

const index = ref(0);
let busy = false;

function go(i: number) {
  if (busy) return;
  const n = items.value.length;
  const target = ((i % n) + n) % n;
  if (target === index.value) return;
  busy = true;
  index.value = target;
  setTimeout(() => (busy = false), 330);
}

// 鼠标悬浮卡片时，滚轮切换
function onWheel(e: WheelEvent) {
  if (e.deltaY > 0) go(index.value + 1);
  else go(index.value - 1);
}
</script>

<template>
  <div class="banner" @wheel.prevent="onWheel">
    <div class="track" :style="{ transform: `translateX(-${index * 100}%)` }">
      <div v-for="(it, i) in items" :key="i" class="slide" :style="{ background: it.color }">
        <span class="slide-title">{{ it.title }}</span>
      </div>
    </div>

    <!-- 底部指示器 -->
    <div class="dots" role="tablist" aria-label="展示卡片">
      <button
        v-for="(_, i) in items"
        :key="i"
        class="dot"
        :class="{ active: i === index }"
        type="button"
        :aria-label="`第 ${i + 1} 张`"
        :aria-selected="i === index"
        role="tab"
        @click="go(i)"
      />
    </div>
  </div>
</template>

<style scoped>
.banner {
  position: relative;
  height: 112px;
  margin-bottom: 18px;
  border-radius: 16px;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.1);
  user-select: none;
  -webkit-user-select: none;
}

.track {
  display: flex;
  height: 100%;
  transition: transform 0.32s cubic-bezier(0.4, 0, 0.2, 1);
}

.slide {
  flex: 0 0 100%;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding-bottom: 30px;
}

.slide-title {
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.04em;
  color: rgba(255, 255, 255, 0.85);
}

.dots {
  position: absolute;
  bottom: 10px;
  left: 0;
  right: 0;
  display: flex;
  justify-content: center;
  gap: 6px;
}

.dot {
  width: 6px;
  height: 6px;
  padding: 0;
  border: none;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.35);
  cursor: pointer;
  transition: transform 0.25s ease, background 0.25s ease;
}

.dot:hover {
  background: rgba(255, 255, 255, 0.6);
}

.dot.active {
  background: #fff;
}

.dot:focus-visible {
  outline: 2px solid rgba(255, 255, 255, 0.8);
  outline-offset: 2px;
}
</style>
