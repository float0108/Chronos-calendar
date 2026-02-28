<script setup lang="ts">
import { Check, Trash2 } from 'lucide-vue-next';
import type { Schedule } from '../types';

const props = defineProps<{
  schedule: Schedule;
}>();

const emit = defineEmits<{
  (e: 'toggle', schedule: Schedule): void;
  (e: 'delete', schedule: Schedule): void;
}>();

function handleToggle(event: Event) {
  event.stopPropagation();
  emit('toggle', props.schedule);
}

function handleDelete(event: Event) {
  event.stopPropagation();
  emit('delete', props.schedule);
}
</script>

<template>
  <div
    class="schedule-item group flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors"
    :class="schedule.is_done ? 'opacity-60' : ''"
    style="font-size: var(--font-size-xs); background-color: var(--schedule-bg);"
    @click.stop
  >
    <button
      @click.stop="handleToggle"
      class="w-3 h-3 rounded border flex items-center justify-center shrink-0 transition-colors"
      :class="schedule.is_done ? 'bg-[var(--primary)] border-[var(--primary)]' : 'border-gray-400 hover:border-gray-600'"
    >
      <Check v-if="schedule.is_done" class="w-2 h-2 text-white" />
    </button>
    <span 
      class="flex-1 truncate"
      :class="schedule.is_done ? 'line-through text-[var(--text-muted)]' : 'text-[var(--text-primary)]'"
    >
      {{ schedule.content }}
    </span>
    <button
      @click.stop="handleDelete"
      class="delete-btn opacity-0 group-hover:opacity-100 p-0.5 hover:bg-red-100 rounded text-red-500 transition-opacity"
    >
      <Trash2 class="w-3 h-3" />
    </button>
  </div>
</template>

<style scoped>
.schedule-item:hover .delete-btn {
  opacity: 1;
}
</style>
