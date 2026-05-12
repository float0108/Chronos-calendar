<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { LayoutList, Plus } from 'lucide-vue-next';
import WindowTitleBar from '../components/WindowTitleBar.vue';
import ListItem from '../components/ListItem.vue';
import SubTaskPanel from '../components/SubTaskPanel.vue';
import { useTheme } from '../composables/useTheme';
import { useTaskOperations } from '../composables/useTaskOperations';
import { saveMainTask, saveSubTask, deleteSchedule, deleteMainTask } from '../api/database';
import { defaultLightSettings } from '../types';
import type { MainTask, Schedule } from '../api/database';
import type { DataChange } from '../types';

// ============ 使用组合式函数 ============
const { settings, themeStyle, cellStyle, loadSettings } = useTheme(defaultLightSettings);
const {
  tasks,
  currentTask,
  subTasks,
  searchKeyword,
  loadTasks,
  loadSubTasks,
  selectTask,
} = useTaskOperations();

// ============ 视图状态 ============
type ViewMode = 'empty' | 'main-list' | 'main-detail' | 'sub-detail';
const viewMode = ref<ViewMode>('empty');
const editingSubTask = ref<Schedule | null>(null);

// ============ 添加任务状态 ============
const isAddingMainTask = ref(false);
const addingKey = ref(0);

// ============ 搜索状态 ============
const isSearchFocused = ref(false);

// ============ 搜索功能：关键词变化时重新加载 ============
watch(searchKeyword, () => {
  loadTasks();
});

// ============ 事件监听清理 ============
const unlisteners: Array<() => void> = [];

// ============ 生命周期 ============
onMounted(async () => {
  loadSettings();

  const unlisten = await listen<DataChange>('schedule-changed', async (event) => {
    const change = event.payload;
    if (change.entity === 'main_task' || change.entity === 'schedule' || change.entity === 'batch') {
      await loadTasks();
      if (currentTask.value?.id) {
        const updatedTask = tasks.value.find(t => t.id === currentTask.value?.id);
        if (updatedTask) {
          currentTask.value = updatedTask;
          await loadSubTasks();
        }
      }
    }
  });
  unlisteners.push(unlisten);

  const unlistenStorage = listen('storage', () => {
    loadSettings();
  });
  unlistenStorage.then(u => unlisteners.push(u));

  await loadTasks();
  await nextTick();
  requestAnimationFrame(async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
  });
});

onUnmounted(() => {
  unlisteners.forEach(fn => fn());
});

// ============ 事件处理 ============
async function handleSelectTask(task: MainTask) {
  await selectTask(task);
  viewMode.value = 'main-list';
}

function handleViewTaskDetail() {
  viewMode.value = 'main-detail';
}

function handleSelectSubTask(subTask: Schedule) {
  editingSubTask.value = subTask;
  viewMode.value = 'sub-detail';
}

function handleBackToList() {
  viewMode.value = 'main-list';
  editingSubTask.value = null;
}

async function handleTasksChanged() {
  await loadTasks();
}

async function handleDeleteTask(taskId: number) {
  await deleteMainTask(taskId);
  if (currentTask.value?.id === taskId) {
    currentTask.value = null;
    subTasks.value = [];
    viewMode.value = 'empty';
  }
  await loadTasks();
}

async function handleSubTasksChanged() {
  await loadSubTasks();
}

async function handleAddSubTask(content: string) {
  if (!currentTask.value?.id) return;
  await saveSubTask(content.trim(), currentTask.value.id);
  await loadSubTasks();
}

async function handleDeleteSubTask(subTaskId: number) {
  await deleteSchedule(subTaskId);
  await loadSubTasks();
}

function handleSearch(keyword: string) {
  searchKeyword.value = keyword;
}

function handleSelectRootTask() {
  if (viewMode.value === 'main-list') {
    handleViewTaskDetail();
  } else {
    viewMode.value = 'main-list';
    editingSubTask.value = null;
  }
}

function handleStartAddTask() {
  viewMode.value = 'main-list';
  addingKey.value++;
  isAddingMainTask.value = true;
}

async function handleAddTask(content: string) {
  const trimmed = content.trim();
  if (!trimmed) {
    isAddingMainTask.value = false;
    addingKey.value = 0;
    return;
  }
  try {
    const newTaskId = await saveMainTask(trimmed);
    await loadTasks();
    if (newTaskId) {
      const newTask = tasks.value.find(t => t.id === newTaskId);
      if (newTask) {
        await selectTask(newTask);
      }
    }
  } catch (error) {
    console.error('Failed to add task:', error);
  }
  isAddingMainTask.value = false;
  addingKey.value = 0;
}

function handleCancelAddTask() {
  isAddingMainTask.value = false;
  addingKey.value = 0;
}
async function handleIconDrag() {
  try {
    const win = getCurrentWindow();
    await win.startDragging();
  } catch (error) {
    console.error('Drag failed:', error);
  }
}
</script>

<template>
  <div class="taskboard-overlay fixed inset-0 z-[9999] flex w-full h-full" :style="themeStyle">
    <div class="taskboard-panel relative flex overflow-hidden w-full h-full rounded-lg transition-colors shadow-lg"
      :style="{
        backgroundColor: 'var(--theme-bg)',
        border: '1px solid var(--theme-border)',
        backdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
        WebkitBackdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
      }">

      <!-- Left Panel: Task List with Title Bar -->
      <div class="left-panel flex flex-col w-80 border-r min-h-0" :style="{ borderColor: 'var(--theme-border)' }">
        <!-- Title Bar using WindowTitleBar component -->
        <WindowTitleBar :theme-style="themeStyle" :hide-close-button="true" @start-drag="handleIconDrag">
          <template #left>
            <button
              class="shrink-0 w-6 h-6 flex items-center justify-center cursor-grab active:cursor-grabbing hover:opacity-80 transition-opacity"
              :style="{ color: themeStyle['--theme-text'] }"
              title="Drag"
              @mousedown.stop="handleIconDrag"
            >
              <LayoutList class="w-4 h-4" />
            </button>
          </template>

          <span v-show="!isSearchFocused"
                class="text-base font-medium leading-relaxed transition-opacity cursor-text"
                :style="{ color: themeStyle['--theme-text'] }"
                @click="isSearchFocused = true">
            Tasks
          </span>
          <input
            v-show="isSearchFocused"
            v-model="searchKeyword"
            type="text"
            placeholder="..."
            class="absolute inset-0 w-full h-full bg-black/5 dark:bg-white/5 rounded-md px-2 outline-none text-sm leading-relaxed text-center selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
            :style="{ color: themeStyle['--theme-text'] }"
            @input="handleSearch(searchKeyword)"
            @focus="isSearchFocused = true"
            @blur="isSearchFocused = false"
          />

          <template #right>
            <button
              @click="handleStartAddTask"
              class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
              :style="{ color: themeStyle['--theme-text'] }"
              title="Add task"
            >
              <Plus class="w-4 h-4" />
            </button>
          </template>
        </WindowTitleBar>

        <!-- Task List -->
        <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
          <div class="space-y-2">
            <!-- Add New Task -->
            <ListItem
              v-if="isAddingMainTask"
              :key="`add-new-task-${addingKey}`"
              is-add-mode
              @add="handleAddTask"
              @cancel="handleCancelAddTask"
              @click.stop
            />

            <!-- Task List Items -->
            <ListItem
              v-for="task in tasks"
              :key="task.id"
              :title="task.content"
              :date="task.create_date"
              :is-done="task.is_done"
              center-calendar
              :selected="currentTask?.id === task.id"
              @update:title="() => {}"
              @update:date="() => {}"
              @delete="handleDeleteTask(task.id!)"
              @toggle-done="handleSelectTask(task)"
              @click="handleSelectTask(task)"
            />
          </div>

          <!-- Empty State -->
          <div v-if="tasks.length === 0" class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity">
            <div class="p-4 rounded-full" :style="{ backgroundColor: 'var(--theme-cell)' }">
              <LayoutList class="w-8 h-8 opacity-20" :style="{ color: themeStyle['--theme-text'] }" />
            </div>
          </div>
        </div>
      </div>

      <!-- Right Panel: SubTask List / Detail -->
      <SubTaskPanel
        :current-task="currentTask"
        :sub-tasks="subTasks"
        :view-mode="viewMode"
        :editing-sub-task="editingSubTask"
        :theme-style="themeStyle"
        :cell-style="cellStyle"
        @select-sub-task="handleSelectSubTask"
        @view-task-detail="handleViewTaskDetail"
        @back-to-list="handleBackToList"
        @select-root-task="handleSelectRootTask"
        @sub-tasks-changed="handleSubTasksChanged"
        @add-sub-task="handleAddSubTask"
        @delete-sub-task="handleDeleteSubTask"
      />
    </div>
  </div>
</template>

<style scoped>
.taskboard-overlay {
  pointer-events: auto;
}

.taskboard-panel {
  pointer-events: auto;
}

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

::selection {
  background-color: var(--theme-primary-alpha);
  color: inherit;
}

input, textarea {
  -webkit-appearance: none;
  appearance: none;
}

input[type="date"]::-webkit-calendar-picker-indicator {
  filter: var(--theme-text-muted);
  cursor: pointer;
}
</style>