<script setup lang="ts">
import { startWindowResize } from '../utils/window';
import type { ResizeDirection } from '../types';

defineProps<{
  isLocked: boolean;
}>();

const handles: { direction: ResizeDirection; class: string }[] = [
  { direction: 'East', class: 'absolute right-0 top-4 bottom-4 w-1 cursor-e-resize z-50 hover:bg-[var(--resize-hover)] transition-colors' },
  { direction: 'West', class: 'absolute left-0 top-4 bottom-4 w-1 cursor-w-resize z-50 hover:bg-[var(--resize-hover)] transition-colors' },
  { direction: 'South', class: 'absolute bottom-0 left-4 right-4 h-1 cursor-s-resize z-50 hover:bg-[var(--resize-hover)] transition-colors' },
  { direction: 'SouthEast', class: 'absolute bottom-0 right-0 w-4 h-4 cursor-se-resize z-50 hover:bg-[var(--resize-hover)] transition-colors' },
];

function handleResize(direction: ResizeDirection, event: MouseEvent, isLocked: boolean) {
  if (isLocked) return;
  event.preventDefault();
  startWindowResize(direction);
}
</script>

<template>
  <div
    v-for="handle in handles"
    :key="handle.direction"
    :class="[handle.class, { 'opacity-30': isLocked, 'pointer-events-none': isLocked }]"
    @mousedown="(e) => handleResize(handle.direction, e, isLocked)"
  ></div>
</template>
