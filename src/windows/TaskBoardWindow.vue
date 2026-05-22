<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import TaskListPanel from '../components/TaskListPanel.vue';
import SubTaskPanel from '../components/SubTaskPanel.vue';
import { useTheme } from '../composables/useTheme';
import { useTaskOperations } from '../composables/useTaskOperations';
import { saveMainTask, saveSubTask, deleteSchedule, deleteMainTask, toggleMainTaskStatus, toggleScheduleStatus, updateScheduleContent, updateScheduleDate, updateScheduleDescription, updateMainTaskContent, updateMainTaskCreateDate, updateMainTaskDescription, updateMainTaskDoneDate } from '../api/database';
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

// ============ 搜索功能：关键词变化时重新加载 ============
const stopSearchWatch = watch(searchKeyword, () => {
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
  stopSearchWatch();
  unlisteners.forEach(fn => fn());
});

// ============ 事件处理 ============
async function handleSelectTask(task: MainTask) {
  await selectTask(task);
  viewMode.value = 'main-list';
}

async function handleToggleMainTaskDone(task: MainTask) {
  if (!task.id) return;
  await toggleMainTaskStatus(task.id, !task.is_done);
  await loadTasks();
}

function handleViewTaskDetail() {
  viewMode.value = 'main-detail';
}

function handleSelectSubTask(subTask: Schedule) {
  editingSubTask.value = subTask;
  viewMode.value = 'sub-detail';
}

async function handleToggleSubTaskDone(subTask: Schedule) {
  if (!subTask.id) return;
  await toggleScheduleStatus(subTask.id, !subTask.is_done);
  await loadSubTasks();
}

async function handleUpdateSubTaskContent(subTaskId: number, content: string) {
  if (!subTaskId) return;
  await updateScheduleContent(subTaskId, content);
  await loadSubTasks();
}

async function handleUpdateMainTaskContent(taskId: number, content: string) {
  if (!taskId) return;
  await updateMainTaskContent(taskId, content);
  await loadTasks();
}

async function handleUpdateSubTaskDate(subTaskId: number, date: string) {
  if (!subTaskId) return;
  await updateScheduleDate(subTaskId, 'create_date', date);
  await loadSubTasks();
}

async function handleUpdateMainTaskDate(taskId: number, date: string) {
  if (!taskId) return;
  await updateMainTaskCreateDate(taskId, date);
  await loadTasks();
}

async function handleUpdateSubTaskDescription(subTaskId: number, description: string) {
  if (!subTaskId) return;
  await updateScheduleDescription(subTaskId, description);
  await loadSubTasks();
}

async function handleUpdateSubTaskDoneDate(subTaskId: number, doneDate: string, isDone: boolean) {
  if (!subTaskId) return;
  await updateScheduleDate(subTaskId, 'done_date', doneDate);
  await toggleScheduleStatus(subTaskId, isDone);
  await loadSubTasks();
}

async function handleUpdateMainTaskDescription(taskId: number, description: string) {
  if (!taskId) return;
  await updateMainTaskDescription(taskId, description);
  await loadTasks();
}

async function handleUpdateMainTaskDoneDate(taskId: number, doneDate: string, isDone: boolean) {
  if (!taskId) return;
  await updateMainTaskDoneDate(taskId, doneDate || null);
  await toggleMainTaskStatus(taskId, isDone);
  await loadTasks();
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
  isAddingMainTask.value = true;
}

async function handleAddTask(content: string) {
  const trimmed = content.trim();
  if (!trimmed) {
    isAddingMainTask.value = false;
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
}

async function handleStartDrag() {
  try {
    const win = getCurrentWindow();
    await win.startDragging();
  } catch (error) {
    console.error('Drag failed:', error);
  }
}

async function handleClose() {
  const win = getCurrentWindow();
  await win.hide();
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

      <!-- Left Panel: Task List -->
      <TaskListPanel
        :tasks="tasks"
        :selected-task="currentTask"
        :theme-style="themeStyle"
        :cell-style="cellStyle"
        :search-keyword="searchKeyword"
        :is-adding-main-task="isAddingMainTask"
        @select-task="handleSelectTask"
        @toggle-done="handleToggleMainTaskDone"
        @start-add-task="handleStartAddTask"
        @add-task="handleAddTask"
        @delete-task="handleDeleteTask"
        @tasks-changed="handleTasksChanged"
        @search="handleSearch"
        @start-drag="handleStartDrag"
      />

      <!-- Right Panel: SubTask List / Detail -->
      <SubTaskPanel
        :current-task="currentTask"
        :sub-tasks="subTasks"
        :view-mode="viewMode"
        :editing-sub-task="editingSubTask"
        :theme-style="themeStyle"
        :cell-style="cellStyle"
        @select-sub-task="handleSelectSubTask"
        @toggle-sub-task-done="handleToggleSubTaskDone"
        @update-sub-task-content="handleUpdateSubTaskContent"
        @update-main-task-content="handleUpdateMainTaskContent"
        @title-saved="handleSubTasksChanged"
        @view-task-detail="handleViewTaskDetail"
        @back-to-list="handleBackToList"
        @select-root-task="handleSelectRootTask"
        @sub-tasks-changed="handleSubTasksChanged"
        @add-sub-task="handleAddSubTask"
        @delete-sub-task="handleDeleteSubTask"
        @update-sub-task-date="handleUpdateSubTaskDate"
        @update-sub-task-description="handleUpdateSubTaskDescription"
        @update-sub-task-done-date="handleUpdateSubTaskDoneDate"
        @update-main-task-date="handleUpdateMainTaskDate"
        @update-main-task-description="handleUpdateMainTaskDescription"
        @update-main-task-done-date="handleUpdateMainTaskDoneDate"
        @close="handleClose"
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