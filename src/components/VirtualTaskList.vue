<!-- components/VirtualTaskList.vue -->
<script setup lang="ts">
import { RecycleScroller } from 'vue-virtual-scroller';
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css';
import ListItem from './ListItem.vue';
import type { MainTask } from '../api/database';

interface Props {
  tasks: MainTask[];
  selectedTaskId: number | null;
  themeStyle: Record<string, string>;
  cellStyle: Record<string, string>;
}

defineProps<Props>();

const emit = defineEmits<{
  'select-task': [task: MainTask];
  'update:title': [task: MainTask, value: string];
  'update:date': [task: MainTask, value: string];
  'toggle-done': [task: MainTask];
  'delete': [taskId: number];
}>();
</script>

<template>
  <RecycleScroller
    :items="tasks"
    :item-size="56"
    key-field="id"
    class="h-full w-full"
    v-slot="{ item: task }"
  >
    <ListItem
      :key="task.id"
      :title="task.content"
      :date="task.create_date"
      :is-done="task.is_done"
      center-calendar
      :selected="selectedTaskId === task.id"
      @update:title="(val) => emit('update:title', task, val)"
      @update:date="(val) => emit('update:date', task, val)"
      @toggle-done="emit('toggle-done', task)"
      @delete="emit('delete', task.id!)"
      @click="emit('select-task', task)"
    />
  </RecycleScroller>
</template>