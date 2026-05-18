<!-- components/TaskListPanel.vue -->
<script setup lang="ts">
import { Plus, GripVertical } from 'lucide-vue-next';
import WindowTitleBar from './WindowTitleBar.vue';
import ListItemPanel from './ListItemPanel.vue';
import type { MainTask } from '../api/database';

interface Props {
  tasks: MainTask[];
  selectedTask: MainTask | null;
  themeStyle: Record<string, string>;
  cellStyle: Record<string, string>;
  searchKeyword?: string;
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

withDefaults(defineProps<Props>(), {
  searchKeyword: '',
});

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
    <!-- Header with WindowTitleBar -->
    <WindowTitleBar
      :theme-style="themeStyle"
      :show-search="true"
      :title="'Tasks'"
      :model-value="searchKeyword"
      search-placeholder="Search tasks..."
      @update:model-value="handleSearch"
      @start-drag="emit('start-drag')"
    >
      <template #left>
        <button
          class="shrink-0 w-5 h-5 flex items-center justify-center cursor-grab active:cursor-grabbing hover:opacity-80 transition-opacity"
          :style="{ color: themeStyle['--theme-text'] }"
          title="Drag"
          @mousedown.stop="emit('start-drag')"
        >
          <GripVertical class="w-3.5 h-3.5" />
        </button>
      </template>

      <template #right>
        <button
          @click="handleStartAdd"
          class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
          :style="{ color: themeStyle['--theme-text'] }"
          title="Add task"
        >
          <Plus class="w-4 h-4" />
        </button>
      </template>
    </WindowTitleBar>

    <!-- Task List -->
    <ListItemPanel
      :items="tasks"
      :selected-item="selectedTask"
      :theme-style="themeStyle"
      :cell-style="cellStyle"
      :is-adding="isAddingMainTask"
      @select="handleSelect"
      @add="handleAdd"
      @delete="handleDelete"
      @toggle-done="handleToggleDone"
      @start-add="emit('start-add-task')"
      @update:is-adding="() => emit('start-add-task')"
    />
  </div>
</template>
