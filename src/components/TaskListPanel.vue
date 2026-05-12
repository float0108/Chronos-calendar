<!-- components/TaskListPanel.vue -->
<script setup lang="ts">
import { ref } from 'vue';
import { LayoutList, Plus } from 'lucide-vue-next';
import ListItem from './ListItem.vue';
import VirtualTaskList from './VirtualTaskList.vue';
import type { MainTask } from '../api/database';

interface Props {
  tasks: MainTask[];
  selectedTask: MainTask | null;
  themeStyle: Record<string, string>;
  cellStyle: Record<string, string>;
  searchKeyword: string;
}

interface Emits {
  'select-task': [task: MainTask];
  'start-add-task': [];
  'tasks-changed': [];
  'search': [keyword: string];
}

defineProps<Props>();
const emit = defineEmits<Emits>();

// 本地状态
const isAddingMainTask = ref(false);
const addingKey = ref(0);
const searchInputRef = ref<HTMLInputElement | null>(null);
const isSearchFocused = ref(false);

// ============ 事件处理 ============

function handleStartAddTask() {
  isAddingMainTask.value = true;
  addingKey.value++;
}

function handleAddTask(content: string) {
  const trimmed = content.trim();
  if (!trimmed) {
    isAddingMainTask.value = false;
    return;
  }
  emit('tasks-changed');
  isAddingMainTask.value = false;
  addingKey.value = 0;
}

function handleCancelAddTask() {
  isAddingMainTask.value = false;
  addingKey.value = 0;
}

function handleSearch(keyword: string) {
  emit('search', keyword);
}

function handleSelectTask(task: MainTask) {
  emit('select-task', task);
}

function handleUpdateTask(_task: MainTask, _val: string) {
  // 占位，实际由父组件处理
}

function handleUpdateDate(_task: MainTask, _val: string) {
  // 占位，实际由父组件处理
}

function handleDeleteTask(_taskId: number) {
  emit('tasks-changed');
}
</script>

<template>
  <div class="left-panel flex flex-col w-80 border-r min-h-0" :style="{ borderColor: cellStyle.borderColor }">
    <!-- Search Header -->
    <div class="flex items-center gap-2 border-b px-3 py-2" :style="{ borderColor: cellStyle.borderColor }">
      <input
        ref="searchInputRef"
        type="text"
        placeholder="Search tasks..."
        :value="searchKeyword"
        @input="(e) => handleSearch((e.target as HTMLInputElement).value)"
        @focus="isSearchFocused = true"
        @blur="isSearchFocused = false"
        class="flex-1 px-2 py-1 rounded text-sm bg-transparent outline-none"
        :style="{
          color: themeStyle['--theme-text'],
          borderColor: isSearchFocused ? themeStyle['--theme-primary'] : 'transparent',
          borderWidth: '1px',
          transition: 'border-color 0.2s',
        }"
      />
      <button
        @click="handleStartAddTask"
        class="p-1.5 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors shrink-0"
        :style="{ color: themeStyle['--theme-text-muted'] }"
        title="Add task"
      >
        <Plus class="w-4 h-4" />
      </button>
    </div>

    <!-- Task List -->
    <div class="flex-1 overflow-hidden custom-scrollbar flex flex-col">
      <!-- Add New Task Mode (outside virtual scroll) -->
      <div v-if="isAddingMainTask" class="px-3 pt-2 pb-1">
        <ListItem
          :key="`add-new-task-${addingKey}`"
          is-add-mode
          @add="handleAddTask"
          @cancel="handleCancelAddTask"
          @click.stop
        />
      </div>

      <!-- Task List with Virtual Scrolling -->
      <div v-if="tasks.length > 0" class="flex-1 overflow-hidden">
        <VirtualTaskList
          :tasks="tasks"
          :selected-task-id="selectedTask?.id ?? null"
          :theme-style="themeStyle"
          :cell-style="cellStyle"
          @select-task="handleSelectTask"
          @update:title="handleUpdateTask"
          @update:date="handleUpdateDate"
          @toggle-done="handleSelectTask"
          @delete="handleDeleteTask"
        />
      </div>

      <!-- Empty State -->
      <div
        v-if="tasks.length === 0 && !isAddingMainTask"
        class="flex flex-col items-center justify-center flex-1 pointer-events-none transition-opacity"
      >
        <div class="p-4 rounded-full" :style="cellStyle">
          <LayoutList
            class="w-8 h-8 opacity-20"
            :style="{ color: themeStyle['--theme-text'] }"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: transparent transparent;
  transition: scrollbar-color 0.3s ease;
}

.custom-scrollbar:hover {
  scrollbar-color: var(--theme-border) transparent;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: transparent;
  border-radius: 4px;
}

.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background-color: var(--theme-border);
}
</style>