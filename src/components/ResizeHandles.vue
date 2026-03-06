<script setup lang="ts">
import type { ResizeDirection } from '../types';
import { startWindowResize } from '../utils/window';

const props = defineProps<{
  isLocked: boolean;
}>();

const handles: { direction: ResizeDirection; class: string }[] = [
  { direction: 'East', class: 'absolute right-0 top-4 bottom-4 w-1 cursor-e-resize z-50 hover:bg-[var(--resize-hover)] transition-colors' },
  { direction: 'West', class: 'absolute left-0 top-4 bottom-4 w-1 cursor-w-resize z-50 hover:bg-[var(--resize-hover)] transition-colors' },
  { direction: 'South', class: 'absolute bottom-0 left-4 right-4 h-1 cursor-s-resize z-50 hover:bg-[var(--resize-hover)] transition-colors' },
  { direction: 'SouthEast', class: 'absolute bottom-0 right-0 w-4 h-4 cursor-se-resize z-50 hover:bg-[var(--resize-hover)] transition-colors' },
];

function handleResize(direction: ResizeDirection, event: MouseEvent) {
  if (props.isLocked) return;
  event.preventDefault();
  startWindowResize(direction);
}
</script>

<template>
  <!-- 锁定模式下不显示调整大小手柄 -->
  <template v-if="!isLocked">
    <div
      v-for="handle in handles"
      :key="handle.direction"
      :class="handle.class"
      @mousedown="(e) => handleResize(handle.direction, e)"
    ></div>
  </template>
</template>
