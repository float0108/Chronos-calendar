<script setup lang="ts">
import { ref, watch } from 'vue';
import type { Schedule } from '../../types';

const props = defineProps<{
  schedule: Schedule | null;
  position: { x: number; y: number; width?: number };
}>();

const tooltipRef = ref<HTMLElement | null>(null);
const scrollTop = ref(0);

// 暴露滚动方法给父组件
function scrollBy(delta: number) {
  if (tooltipRef.value) {
    tooltipRef.value.scrollTop += delta;
  }
}

// 重置滚动位置当内容变化时
watch(() => props.schedule, () => {
  scrollTop.value = 0;
  if (tooltipRef.value) {
    tooltipRef.value.scrollTop = 0;
  }
});

defineExpose({ scrollBy });
</script>

<template>
  <Teleport to="body">
    <div
      v-if="props.schedule && props.schedule.description"
      ref="tooltipRef"
      class="fixed z-[9999] max-h-48 px-3 py-2 text-xs border rounded-lg shadow-lg overflow-y-auto tooltip-scroll pointer-events-none tooltip-theme"
      :style="{
        left: props.position.x + 'px',
        top: props.position.y + 'px',
        width: (props.position.width || 240) + 'px',
      }"
    >
      <div class="font-medium mb-1 tooltip-title">{{ props.schedule.content }}</div>
      <div class="whitespace-pre-wrap tooltip-desc">{{ props.schedule.description }}</div>
    </div>
  </Teleport>
</template>

<style scoped>
.tooltip-theme {
  background-color: var(--solid-bg);
  border-color: var(--border-light);
  color: var(--text-primary);
}

.tooltip-title {
  color: var(--text-primary);
}

.tooltip-desc {
  color: var(--text-muted);
  word-break: normal;
  overflow-wrap: break-word;
}

.tooltip-scroll {
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
}

.tooltip-scroll::-webkit-scrollbar {
  width: 4px;
}

.tooltip-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.tooltip-scroll::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.5);
  border-radius: 2px;
}

.tooltip-scroll::-webkit-scrollbar-thumb:hover {
  background-color: rgba(156, 163, 175, 0.7);
}
</style>
