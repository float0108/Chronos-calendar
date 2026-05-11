<script setup lang="ts">
import { ref, onMounted, nextTick, computed, onUnmounted, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { LayoutList, ListTodo, Plus, X, Info } from 'lucide-vue-next';
import ListItem from '../components/ListItem.vue';
import ScheduleEditor from '../components/ScheduleEditor.vue';
import WindowTitleBar from '../components/WindowTitleBar.vue';
import {
  loadMainTasks,
  searchMainTasks,
  saveMainTask,
  updateMainTaskContent,
  updateMainTaskCreateDate,
  toggleMainTaskStatus,
  deleteMainTask,
  loadSchedulesByFatherTask,
  saveSubTask,
  deleteSchedule,
  toggleScheduleStatus,
  updateScheduleContent,
  updateScheduleDescription,
  updateScheduleDate,
  updateMainTaskDescription,
  updateMainTaskDoneDate,
  type MainTask,
  type Schedule,
} from '../api/database';
import { hexToRgba, adjustBrightness } from '../utils/color';
import type { AppSettings, DataChange } from '../types';
import { defaultLightSettings, defaultDarkSettings } from '../types';

const settings = ref<AppSettings>({ ...defaultLightSettings });
const tasks = ref<MainTask[]>([]);
const currentTask = ref<MainTask | null>(null);
const subTasks = ref<Schedule[]>([]);
const searchKeyword = ref('');

const effectiveTheme = computed(() => {
  if (settings.value.theme_mode === 'system') {
    return document.documentElement.getAttribute('data-theme') as 'light' | 'dark' || 'light';
  }
  return settings.value.theme_mode;
});

const themeStyle = computed(() => {
  const s = settings.value;
  const bgOpacity = s.bg_opacity / 100;
  const cellOpacity = s.cell_opacity / 100;
  const theme = effectiveTheme.value;
  return {
    '--theme-bg': hexToRgba(s.bg_color, bgOpacity),
    '--theme-cell': hexToRgba(s.cell_color, cellOpacity),
    '--theme-text': s.text_color,
    '--theme-text-secondary': adjustBrightness(s.text_color, 30),
    '--theme-text-muted': s.muted_text_color,
    '--theme-primary': s.primary_color,
    '--theme-primary-alpha': hexToRgba(s.primary_color, 0.2),
    '--theme-border': s.cell_border_color || (theme === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.05)'),
    '--theme-font-family': s.font_family,
    '--theme-font-size': `${s.font_size}px`,
    'font-family': s.font_family,
    'font-size': `${s.font_size}px`,
  };
});

const cellStyle = computed(() => ({
  backgroundColor: 'var(--theme-cell)',
  borderColor: 'var(--theme-border)',
}));

// 视图模式：统一右侧分栏，无 modal
type ViewMode = 'empty' | 'main-list' | 'main-detail' | 'sub-detail';
const viewMode = ref<ViewMode>('empty');
const isAddingMainTask = ref(false);  // 左侧面板：添加主任务
const isAddingSubTask = ref(false);   // 右侧面板：添加子任务
const addingKey = ref(0); // 用于强制渲染新增项
const editingSubTask = ref<Schedule | null>(null);

// 搜索框焦点
const isSearchFocused = ref(false);
const searchInputRef = ref<HTMLInputElement | null>(null);

// 详情编辑
const editDescription = ref('');
const editCreateDate = ref('');
const editDoneDate = ref('');
const scheduleEditorRef = ref<InstanceType<typeof ScheduleEditor> | null>(null);

// 任务详情编辑
const taskEditDescription = ref('');
const taskEditCreateDate = ref('');
const taskEditDoneDate = ref('');

// Debounce timer for auto-save
let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;

// Auto-save task detail when fields change
watch([taskEditDescription, taskEditCreateDate, taskEditDoneDate], () => {
  if (viewMode.value !== 'main-detail' || !currentTask.value?.id) return;
  if (autoSaveTimer) clearTimeout(autoSaveTimer);
  autoSaveTimer = setTimeout(() => {
    handleSaveTaskDetail();
  }, 1000);
});

function loadSettings() {
  const saved = localStorage.getItem('chronos_settings');
  if (saved) {
    const parsed = JSON.parse(saved);
    const actualTheme = parsed.theme_mode === 'system'
      ? (document.documentElement.getAttribute('data-theme') as 'light' | 'dark' || 'light')
      : (parsed.theme_mode || 'light');
    const defaults = actualTheme === 'dark' ? defaultDarkSettings : defaultLightSettings;
    settings.value = { ...defaults, ...parsed };
  }
  applyTheme();
}

function applyTheme() {
  const s = settings.value;
  const root = document.documentElement;
  root.style.setProperty('--primary', s.primary_color);
  root.style.setProperty('--text-primary', s.text_color);
  root.style.setProperty('--text-muted', s.muted_text_color);
}

function handleSettingsUpdate() {
  loadSettings();
}

async function loadTasks() {
  try {
    if (searchKeyword.value.trim()) {
      tasks.value = await searchMainTasks(searchKeyword.value);
    } else {
      tasks.value = await loadMainTasks();
    }

    // Refresh currentTask if it exists
    if (currentTask.value?.id) {
      const updated = tasks.value.find(t => t.id === currentTask.value!.id);
      if (updated) {
        currentTask.value = updated;
      } else {
        // Task was deleted, clear selection
        currentTask.value = null;
        subTasks.value = [];
        viewMode.value = 'empty';
      }
    }
  } catch (error) {
    console.error('Failed to load tasks:', error);
    tasks.value = [];
  }
}

async function loadSubTasks() {
  if (!currentTask.value?.id) {
    subTasks.value = [];
    return;
  }
  subTasks.value = await loadSchedulesByFatherTask(currentTask.value.id);
}

// 左侧面板：添加主任务
function handleStartAddMainTask() {
  viewMode.value = 'main-list';
  addingKey.value++;
  isAddingMainTask.value = true;
}

// 右侧面板：添加子任务
function handleStartAddSubTask() {
  viewMode.value = 'main-list';
  addingKey.value++;
  isAddingSubTask.value = true;
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

function handleCancelAddMainTask() {
  isAddingMainTask.value = false;
  addingKey.value = 0;
}

async function handleAddSubTask(content: string) {
  if (!currentTask.value?.id) return;
  await saveSubTask(content.trim(), currentTask.value.id);
  await loadSubTasks();
  isAddingSubTask.value = false;
  addingKey.value = 0;
}

function handleCancelAddSubTask() {
  isAddingSubTask.value = false;
  addingKey.value = 0;
}

async function handleToggleDone(task: MainTask) {
  if (!task.id) return;
  try {
    await toggleMainTaskStatus(task.id, !task.is_done);
    await loadTasks();
  } catch (error) {
    console.error('Failed to toggle task:', error);
  }
}

async function handleDeleteTask(taskId: number) {
  try {
    await deleteMainTask(taskId);
    if (currentTask.value?.id === taskId) {
      currentTask.value = null;
      subTasks.value = [];
      viewMode.value = 'empty';
    }
    await loadTasks();
  } catch (error) {
    console.error('Failed to delete task:', error);
  }
}

async function handleUpdateTask(task: MainTask, newContent: string) {
  if (!task.id) return;
  const trimmed = newContent.trim();
  if (!trimmed) {
    await handleDeleteTask(task.id);
    return;
  }
  if (trimmed === task.content) return;
  try {
    await updateMainTaskContent(task.id, trimmed);
    await loadTasks();
  } catch (error) {
    console.error('Failed to update task:', error);
  }
}

async function handleUpdateTaskDate(task: MainTask, newDate: string) {
  if (!task.id) return;
  try {
    await updateMainTaskCreateDate(task.id, newDate);
    await loadTasks();
  } catch (error) {
    console.error('Failed to update task date:', error);
  }
}

async function selectTask(task: MainTask) {
  currentTask.value = task;
  await loadSubTasks();
  viewMode.value = 'main-list';
  initTaskEditData();
}

function initTaskEditData() {
  if (!currentTask.value) return;
  taskEditDescription.value = currentTask.value.description || '';
  taskEditCreateDate.value = currentTask.value.create_date || '';
  taskEditDoneDate.value = currentTask.value.done_date || '';
}

function handleViewTaskDetail() {
  viewMode.value = 'main-detail';
  initTaskEditData();
}

function handleBackToList() {
  viewMode.value = 'main-list';
  setTimeout(() => {
    editingSubTask.value = null;
  }, 150);
}

function handleSelectSubTask(subTask: Schedule) {
  editingSubTask.value = subTask;
  editDescription.value = subTask.description || '';
  editCreateDate.value = subTask.create_date || '';
  editDoneDate.value = subTask.done_date || '';
  viewMode.value = 'sub-detail';
  nextTick(() => scheduleEditorRef.value?.loadTasks());
}

async function handleSaveDetail() {
  if (!editingSubTask.value) return;

  if (editDescription.value !== (editingSubTask.value.description || '')) {
    await updateScheduleDescription(editingSubTask.value.id!, editDescription.value);
  }
  if (editCreateDate.value !== (editingSubTask.value.create_date || '')) {
    await updateScheduleDate(editingSubTask.value.id!, 'create_date', editCreateDate.value);
  }
  if (editDoneDate.value !== (editingSubTask.value.done_date || '')) {
    await updateScheduleDate(editingSubTask.value.id!, 'done_date', editDoneDate.value);
  }
  if (editDoneDate.value && !editingSubTask.value.is_done) {
    await toggleScheduleStatus(editingSubTask.value.id!, true);
  }

  await loadSubTasks();
  handleBackToList();
}

async function handleSaveTaskDetail() {
  if (!currentTask.value?.id) return;
  const originalTask = tasks.value.find(t => t.id === currentTask.value?.id);
  if (!originalTask) return;

  if (taskEditDescription.value !== (originalTask.description || '')) {
    await updateMainTaskDescription(currentTask.value.id, taskEditDescription.value);
  }
  if (taskEditCreateDate.value !== (originalTask.create_date || '')) {
    await updateMainTaskCreateDate(currentTask.value.id, taskEditCreateDate.value);
  }
  const currentDoneDate = originalTask.done_date || '';
  if (taskEditDoneDate.value !== currentDoneDate) {
    await updateMainTaskDoneDate(currentTask.value.id, taskEditDoneDate.value || null);
  }
  if (taskEditDoneDate.value && !originalTask.is_done) {
    await toggleMainTaskStatus(currentTask.value.id, true);
  }

  await loadTasks();
}

async function handleSaveTaskDetailAndBack() {
  await handleSaveTaskDetail();
  handleBackToList();
}

// 子任务操作
async function handleToggleSubTaskDone(subTask: Schedule) {
  if (!subTask.id) return;
  await toggleScheduleStatus(subTask.id, !subTask.is_done);
  await loadSubTasks();
}

async function handleDeleteSubTask(subTaskId: number) {
  await deleteSchedule(subTaskId);
  await loadSubTasks();
}

async function handleUpdateSubTaskContent(subTask: Schedule, newContent: string) {
  if (!subTask.id) return;
  const trimmed = newContent.trim();
  if (!trimmed) {
    await handleDeleteSubTask(subTask.id);
    return;
  }
  if (trimmed === subTask.content) return;
  await updateScheduleContent(subTask.id, trimmed);
  await loadSubTasks();
}

async function handleUpdateSubTaskDate(subTask: Schedule, newDate: string) {
  if (!subTask.id) return;
  await updateScheduleDate(subTask.id, 'create_date', newDate);
  await loadSubTasks();
}

async function handleClose() {
  const win = getCurrentWindow();
  await win.hide();
}

async function handleIconDrag() {
  try {
    const win = getCurrentWindow();
    await win.startDragging();
  } catch (error) {
    console.error('Drag failed:', error);
  }
}

let unlisten: (() => void) | undefined;
let unlistenScheduleChange: (() => void) | undefined;

onMounted(async () => {
  loadSettings();

  unlisten = await listen<DataChange>('schedule-changed', async (event) => {
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

  window.addEventListener('storage', handleSettingsUpdate);

  await loadTasks();
  await nextTick();
  requestAnimationFrame(async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
  });
});

onUnmounted(() => {
  unlisten?.();
  unlistenScheduleChange?.();
  window.removeEventListener('storage', handleSettingsUpdate);
});
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
      <div class="left-panel flex flex-col border-r" :style="{ borderColor: 'var(--theme-border)', width: '280px', minWidth: '200px' }">
        <WindowTitleBar
          :theme-style="themeStyle"
          @close="handleClose"
          @start-drag="handleIconDrag"
        >
          <template #left>
            <button @mousedown.stop="handleIconDrag"
              class="shrink-0 w-6 h-6 flex items-center justify-center cursor-grab active:cursor-grabbing hover:opacity-80 transition-opacity"
              :style="{ color: 'var(--theme-text)' }"
              title="Drag">
              <LayoutList class="w-4 h-4" />
            </button>
          </template>

          <span v-show="!isSearchFocused"
            class="text-base font-medium leading-relaxed transition-opacity"
            :style="{ color: 'var(--theme-text)' }"
            @click="isSearchFocused = true">
            Tasks
          </span>
          <input
            ref="searchInputRef"
            v-show="isSearchFocused"
            v-model="searchKeyword"
            type="text"
            placeholder="..."
            class="absolute inset-0 w-full h-full bg-black/5 dark:bg-white/5 rounded-md pl-2 pr-16 outline-none text-sm leading-relaxed text-center selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
            :style="{ color: 'var(--theme-text)' }"
            @input="loadTasks"
            @focus="isSearchFocused = true"
            @blur="isSearchFocused = false"
            @mousedown.stop
          />
          <button
            v-if="searchKeyword.trim() && isSearchFocused"
            @click="searchKeyword = ''; loadTasks()"
            class="absolute right-12 top-1/2 -translate-y-1/2 w-5 h-5 flex items-center justify-center rounded hover:bg-black/10 dark:hover:bg-white/10"
            :style="{ color: 'var(--theme-text-muted)' }"
          >
            <X class="w-3.5 h-3.5" />
          </button>
          <span
            v-if="searchKeyword.trim() && isSearchFocused"
            class="absolute right-2 top-1/2 -translate-y-1/2 text-xs px-1.5 py-0.5 rounded"
            :style="{ color: 'var(--theme-text-muted)', backgroundColor: 'var(--theme-cell)' }"
          >
            {{ tasks.length }}
          </span>

          <template #right>
            <button @click="handleStartAddMainTask"
              class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
              :style="{ color: 'var(--theme-text)' }">
              <Plus class="w-4 h-4" />
            </button>
          </template>
        </WindowTitleBar>

        <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
          <div class="space-y-2">
            <ListItem
              v-if="isAddingMainTask"
              key="add-new-task"
              is-add-mode
              @add="handleAddTask"
              @cancel="handleCancelAddMainTask"
              @click.stop
            />

            <ListItem
              v-for="task in tasks"
              :key="task.id"
              :title="task.content"
              :date="task.create_date"
              :is-done="task.is_done"
              center-calendar
              :selected="currentTask?.id === task.id"
              @update:title="(val) => handleUpdateTask(task, val)"
              @update:date="(val) => handleUpdateTaskDate(task, val)"
              @delete="handleDeleteTask(task.id!)"
              @toggle-done="handleToggleDone(task)"
              @click="selectTask(task)"
            />
          </div>

          <div v-if="tasks.length === 0 && !isAddingMainTask" class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity">
            <div class="p-4 rounded-full" :style="{ backgroundColor: 'var(--theme-cell)' }">
              <LayoutList class="w-8 h-8 opacity-20" :style="{ color: 'var(--theme-text)' }" />
            </div>
          </div>
        </div>
      </div>

      <!-- Right Panel: Subtask List / Detail -->
      <div class="right-panel flex flex-col flex-1 min-w-0">
        <!-- Header with breadcrumb navigation -->
        <div class="flex items-center border-b px-3 py-2" :style="{ borderColor: 'var(--theme-border)' }">
          <template v-if="currentTask">
            <!-- Breadcrumb navigation - flex-shrink first, truncate only when truly needed -->
            <nav class="flex items-center gap-1 text-sm min-w-0 flex-1">
              <!-- Root: always shown -->
              <button
                @click="selectTask(currentTask!)"
                class="px-2 py-1 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors shrink-0"
                :style="{ color: viewMode === 'main-list' ? 'var(--theme-text)' : 'var(--theme-text-muted)' }"
                :title="currentTask.content"
              >
                <span class="truncate max-w-[150px] inline-block align-bottom">{{ currentTask.content }}</span>
              </button>

              <template v-if="viewMode === 'main-detail'">
                <span class="text-[var(--theme-text-muted)] shrink-0">/</span>
                <span class="px-2 py-1 shrink-0" :style="{ color: 'var(--theme-text)' }">Info</span>
              </template>

              <template v-if="viewMode === 'sub-detail' && editingSubTask">
                <span class="text-[var(--theme-text-muted)] shrink-0">/</span>
                <span class="px-2 py-1 truncate max-w-[150px] inline-block align-bottom" :style="{ color: 'var(--theme-text)' }">
                  {{ editingSubTask.content }}
                </span>
              </template>
            </nav>

            <!-- Add subtask button -->
            <div v-if="viewMode === 'main-list'" class="ml-auto flex items-center">
              <button
                @click="handleStartAddSubTask"
                class="p-1.5 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors"
                :style="{ color: 'var(--theme-text-muted)' }"
                title="Add subtask"
              >
                <Plus class="w-4 h-4" />
              </button>
            </div>
          </template>
          <template v-else>
            <span class="text-base font-medium" :style="{ color: 'var(--theme-text-muted)' }">
              Select a task
            </span>
          </template>
        </div>

        <!-- Content Area -->
        <div class="flex-1 relative overflow-hidden">
          <!-- Subtask list view -->
          <div v-if="viewMode === 'main-list'" class="absolute inset-0 flex flex-col w-full h-full">
            <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
              <div class="space-y-2">
                <!-- Main task entry button -->
                <div
                  class="group flex items-center gap-2 px-3 py-2 rounded-lg transition-all cursor-pointer hover:bg-black/5 dark:hover:bg-white/5"
                  :style="cellStyle"
                  @click="handleViewTaskDetail"
                >
                  <Info class="w-4 h-4 opacity-60 group-hover:opacity-100 transition-opacity" :style="{ color: themeStyle['--theme-text-muted'] }" />
                  <span class="text-sm opacity-60 group-hover:opacity-100 transition-opacity" :style="{ color: themeStyle['--theme-text-muted'] }">Task Info</span>
                </div>

                <!-- Add new subtask -->
                <ListItem
                  v-if="isAddingSubTask"
                  is-add-mode
                  @add="handleAddSubTask"
                  @cancel="handleCancelAddSubTask"
                  @click.stop
                />

                <!-- Subtask list -->
                <ListItem
                  v-for="subTask in subTasks"
                  :key="subTask.id"
                  :title="subTask.content"
                  :preview="subTask.description"
                  :date="subTask.create_date"
                  :is-done="subTask.is_done"
                  center-calendar
                  @update:title="(val) => handleUpdateSubTaskContent(subTask, val)"
                  @update:date="(val) => handleUpdateSubTaskDate(subTask, val)"
                  @delete="handleDeleteSubTask(subTask.id!)"
                  @toggle-done="handleToggleSubTaskDone(subTask)"
                  @click="handleSelectSubTask(subTask)"
                />
              </div>

              <div v-if="subTasks.length === 0 && !isAddingSubTask" class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity">
                <div class="p-4 rounded-full" :style="cellStyle">
                  <ListTodo class="w-8 h-8 opacity-20" :style="{ color: themeStyle['--theme-text'] }" />
                </div>
              </div>
            </div>
          </div>

          <!-- Subtask detail view -->
          <div v-else-if="viewMode === 'sub-detail'" class="absolute inset-0 flex flex-col w-full h-full">
            <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar p-3">
              <div class="h-full rounded-lg p-3" :style="cellStyle">
                <ScheduleEditor
                  ref="scheduleEditorRef"
                  class="h-full"
                  v-model:description="editDescription"
                  v-model:create-date="editCreateDate"
                  v-model:done-date="editDoneDate"
                  :show-content="false"
                  :show-father-task="true"
                  :editable-father-task="false"
                  :father-task-id="currentTask?.id"
                  @save="handleSaveDetail"
                  @cancel="handleBackToList"
                />
              </div>
            </div>
          </div>

          <!-- Task detail view -->
          <div v-else-if="viewMode === 'main-detail'" class="absolute inset-0 flex flex-col w-full h-full">
            <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar p-3">
              <div class="h-full rounded-lg p-3" :style="cellStyle">
                <ScheduleEditor
                  class="h-full"
                  v-model:description="taskEditDescription"
                  v-model:create-date="taskEditCreateDate"
                  v-model:done-date="taskEditDoneDate"
                  :show-content="false"
                  :show-father-task="false"
                  @save="handleSaveTaskDetailAndBack"
                  @cancel="handleBackToList"
                />
              </div>
            </div>
          </div>

          <!-- Empty state: no task selected -->
          <div v-else class="absolute inset-0 flex flex-col items-center justify-center">
            <div class="p-4 rounded-full mb-4" :style="cellStyle">
              <ListTodo class="w-10 h-10 opacity-20" :style="{ color: 'var(--theme-text)' }" />
            </div>
            <p class="text-sm" :style="{ color: 'var(--theme-text-muted)' }">
              Select a task to view its schedules
            </p>
          </div>
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