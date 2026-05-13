<!-- components/TaskListPanel.vue -->
<script setup lang="ts">
import { Plus, GripVertical } from 'lucide-vue-next';
import ListItemPanel from './ListItemPanel.vue';
import type { MainTask } from '../api/database';

interface Props {
  tasks: MainTask[];
  selectedTask: MainTask | null;
  themeStyle: Record<string, string>;
  cellStyle: Record<string, string>;
  searchKeyword: string;
  isAddingMainTask?: boolean;
}

interface Emits {
  'select-task': [task: MainTask];
  'start-add-task': [];
  'add-task': [content: string];
  'delete-task': [taskId: number];
  'tasks-changed': [];
  'search': [keyword: string];
  'start-drag': [];
}

defineProps<Props>();
const emit = defineEmits<Emits>();

function handleSelect(task: MainTask) {
  emit('select-task', task);
}

function handleAdd(content: string) {
  emit('add-task', content);
}

function handleDelete(taskId: number) {
  emit('delete-task', taskId);
}

function handleToggleDone(task: MainTask) {
  emit('select-task', task);
}

function handleSearch(keyword: string) {
  emit('search', keyword);
}

function handleStartAdd() {
  emit('start-add-task');
}
</script>

<template>
  <div class="left-panel flex flex-col w-80 border-r min-h-0" :style="{ borderColor: cellStyle.borderColor }">
    <!-- Header with Drag, Search and Add -->
    <div class="flex items-center gap-2 border-b px-3 py-2 shrink-0 group" :style="{ borderColor: cellStyle.borderColor }">
      <button
        class="shrink-0 w-5 h-5 flex items-center justify-center cursor-grab active:cursor-grabbing hover:opacity-80 transition-opacity"
        :style="{ color: themeStyle['--theme-text'] }"
        title="Drag"
        @mousedown.stop="emit('start-drag')"
      >
        <GripVertical class="w-3.5 h-3.5" />
      </button>
      <input
        type="text"
        placeholder="Search tasks..."
        :value="searchKeyword"
        @input="(e) => handleSearch((e.target as HTMLInputElement).value)"
        class="flex-1 px-2 py-1 rounded text-sm bg-transparent outline-none"
        :style="{
          color: themeStyle['--theme-text'],
        }"
      />
      <button
        @click="handleStartAdd"
        class="p-1 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors shrink-0"
        :style="{ color: themeStyle['--theme-text-muted'] }"
        title="Add task"
      >
        <Plus class="w-4 h-4" />
      </button>
    </div>

    <!-- Task List -->
    <ListItemPanel
      :items="tasks"
      :selected-item="selectedTask"
      :theme-style="themeStyle"
      :cell-style="cellStyle"
      :search-keyword="searchKeyword"
      :is-adding="isAddingMainTask"
      :show-header="false"
      @select="handleSelect"
      @add="handleAdd"
      @delete="handleDelete"
      @toggle-done="handleToggleDone"
      @search="handleSearch"
      @start-add="emit('start-add-task')"
      @update:is-adding="() => emit('start-add-task')"
    />
  </div>
</template>
