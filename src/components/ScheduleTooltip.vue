<script setup lang="ts">
import { ref, onUnmounted } from 'vue';
import type { Schedule } from '../types';

const props = defineProps<{
  schedule: Schedule | null;
  position: { x: number; y: number };
}>();

const tooltipRef = ref<HTMLElement | null>(null);

function handleWheel(event: WheelEvent) {
  if (tooltipRef.value) {
    const tooltip = tooltipRef.value;
    if (tooltip.scrollHeight > tooltip.clientHeight) {
      event.preventDefault();
      tooltip.scrollTop += event.deltaY;
    }
  }
}

onUnmounted(() => {
  // 清理工作在这里进行
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="props.schedule && props.schedule.description"
      ref="tooltipRef"
      class="fixed z-[9999] max-w-xs max-h-48 px-3 py-2 text-xs bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg overflow-y-auto tooltip-scroll pointer-events-none"
      :style="{
        left: props.position.x + 'px',
        top: props.position.y + 'px'
      }"
      @wheel="handleWheel"
    >
      <div class="font-medium text-gray-900 dark:text-gray-100 mb-1">{{ props.schedule.content }}</div>
      <div class="text-gray-600 dark:text-gray-400 whitespace-pre-wrap">{{ props.schedule.description }}</div>
    </div>
  </Teleport>
</template>

<style scoped>
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
