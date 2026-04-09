<script setup lang="ts">
import { ref, onMounted, nextTick, computed, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { emit } from '@tauri-apps/api/event';
import { X, Trash2, ListTodo, Plus, ChevronLeft } from 'lucide-vue-next';
import ListItem from '../components/ListItem.vue';
import {
  loadSchedulesByFatherTask,
  saveSubTask,
  deleteSchedule,
  toggleScheduleStatus,
  toggleMainTaskStatus,
  updateScheduleContent,
  updateScheduleDescription,
  updateScheduleDate,
  loadMainTasks,
  updateMainTaskContent,
  updateMainTaskDescription,
  updateMainTaskCreateDate,
  updateMainTaskDoneDate,
  type Schedule,
  type MainTask
} from '../composables/db';
import { hexToRgba, adjustBrightness } from '../utils/color';
import ScheduleEditor from '../components/ScheduleEditor.vue';
import type { AppSettings } from '../types';
import { defaultLightSettings, defaultDarkSettings } from '../types';

const settings = ref<AppSettings>({ ...defaultLightSettings });
const tasks = ref<MainTask[]>([]);
const currentTask = ref<MainTask | null>(null);
const subTasks = ref<Schedule[]>([]);

// 实际主题（解析 system）
const effectiveTheme = computed(() => {
  if (settings.value.theme_mode === 'system') {
    // 从 data-theme 属性获取当前实际主题（由主窗口维护）
    return document.documentElement.getAttribute('data-theme') as 'light' | 'dark' || 'light';
  }
  return settings.value.theme_mode;
});

// 视图模式: 'task' = 主任务详情, 'list' = 子任务列表, 'detail' = 子任务编辑
const viewMode = ref<'task' | 'list' | 'detail'>('list');

// 新增模式
const isAdding = ref(false);

// 子任务编辑状态
const editingSubTask = ref<Schedule | null>(null);

// 标题编辑状态
const isEditingTitle = ref(false);
const editingTitle = ref('');
const titleInputRef = ref<HTMLInputElement | null>(null);

// 编辑器数据
const editDescription = ref('');
const editCreateDate = ref('');
const editDoneDate = ref('');
const scheduleEditorRef = ref<InstanceType<typeof ScheduleEditor> | null>(null);

// 主任务编辑数据
const taskEditDescription = ref('');
const taskEditCreateDate = ref('');
const taskEditDoneDate = ref('');

// 动态主题样式
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
  // 不覆盖 data-theme，由主窗口维护
}

function handleSettingsUpdate() {
  loadSettings();
}

async function loadSubTasks() {
  if (!currentTask.value?.id) {
    subTasks.value = [];
    return;
  }
  subTasks.value = await loadSchedulesByFatherTask(currentTask.value.id);
}

// 通知主窗口刷新数据
async function notifyMainToRefresh() {
  try {
    await emit('schedule-changed', {});
  } catch (error) {
    console.error('Failed to notify main window:', error);
  }
}

function handleStartAdding() {
  isAdding.value = true;
}

async function handleAddSubTask(content: string) {
  const trimmed = content.trim();
  if (!trimmed || !currentTask.value?.id) {
    isAdding.value = false;
    return;
  }

  try {
    await saveSubTask(trimmed, currentTask.value.id);
    await loadSubTasks();
    await notifyMainToRefresh();
  } catch (error) {
    console.error('Failed to add sub task:', error);
  }
  isAdding.value = false;
}

function handleCancelAdd() {
  isAdding.value = false;
}

async function handleToggleDone(subTask: Schedule) {
  if (!subTask.id) return;
  try {
    await toggleScheduleStatus(subTask.id, !subTask.is_done);
    await loadSubTasks();
    await notifyMainToRefresh();
  } catch (error) {
    console.error('Failed to toggle sub task:', error);
  }
}

async function handleDeleteSubTask(subTaskId: number) {
  try {
    await deleteSchedule(subTaskId);
    await loadSubTasks();
    await notifyMainToRefresh();
  } catch (error) {
    console.error('Failed to delete sub task:', error);
  }
}

async function handleUpdateSubTask(subTask: Schedule, newContent: string) {
  if (!subTask.id) return;
  const trimmed = newContent.trim();
  if (!trimmed) {
    await handleDeleteSubTask(subTask.id);
    return;
  }
  if (trimmed === subTask.content) return;
  try {
    await updateScheduleContent(subTask.id, trimmed);
    await loadSubTasks();
    await notifyMainToRefresh();
  } catch (error) {
    console.error('Failed to update sub task:', error);
  }
}

async function handleUpdateSubTaskDate(subTask: Schedule, newDate: string) {
  if (!subTask.id) return;
  try {
    await updateScheduleDate(subTask.id, 'create_date', newDate);
    await loadSubTasks();
    await notifyMainToRefresh();
  } catch (error) {
    console.error('Failed to update sub task date:', error);
  }
}

// 点击进入主任务详情
function handleViewTaskDetail() {
  isEditingTitle.value = false;
  viewMode.value = 'task';
}

// 开始编辑标题
function startEditingTitle(defaultTitle: string) {
  editingTitle.value = defaultTitle;
  isEditingTitle.value = true;
  nextTick(() => {
    titleInputRef.value?.focus();
    titleInputRef.value?.select();
  });
}

// 保存主任务标题
async function saveTaskTitle() {
  if (!currentTask.value?.id) return;
  const trimmed = editingTitle.value.trim();
  if (trimmed && trimmed !== currentTask.value.content) {
    try {
      await updateMainTaskContent(currentTask.value.id, trimmed);
      currentTask.value.content = trimmed;
      await notifyMainToRefresh();
    } catch (error) {
      console.error('Failed to update task title:', error);
    }
  }
  isEditingTitle.value = false;
}

// 取消编辑标题
function cancelEditingTitle() {
  isEditingTitle.value = false;
  editingTitle.value = '';
}

// 返回主任务详情
function handleBackToTask() {
  isEditingTitle.value = false;
  viewMode.value = 'task';
}

// 点击条目进入子任务详情
function handleSelectSubTask(subTask: Schedule) {
  isEditingTitle.value = false;
  editingSubTask.value = subTask;
  editDescription.value = subTask.description || '';
  editCreateDate.value = subTask.create_date || '';
  editDoneDate.value = subTask.done_date || '';
  viewMode.value = 'detail';
  // 确保任务列表已加载
  nextTick(() => scheduleEditorRef.value?.loadTasks());
}

// 返回子任务列表
function handleBackToList() {
  isEditingTitle.value = false;
  viewMode.value = 'list';
  setTimeout(() => {
    editingSubTask.value = null;
  }, 150);
}

// 保存子任务详情
async function handleSaveDetail() {
  if (!editingSubTask.value?.id) return;

  try {
    // 更新描述
    if (editDescription.value !== (editingSubTask.value.description || '')) {
      await updateScheduleDescription(editingSubTask.value.id, editDescription.value);
    }

    // 更新创建日期
    if (editCreateDate.value !== (editingSubTask.value.create_date || '')) {
      await updateScheduleDate(editingSubTask.value.id, 'create_date', editCreateDate.value);
    }

    // 更新完成日期
    if (editDoneDate.value !== (editingSubTask.value.done_date || '')) {
      await updateScheduleDate(editingSubTask.value.id, 'done_date', editDoneDate.value);
    }

    // 如果完成日期非空，自动设置为 done 状态
    if (editDoneDate.value && !editingSubTask.value.is_done) {
      await toggleScheduleStatus(editingSubTask.value.id, true);
    }

    await loadSubTasks();
    await notifyMainToRefresh();
    handleBackToList();
  } catch (error) {
    console.error('Failed to save detail:', error);
  }
}

// 初始化主任务编辑数据
function initTaskEditData() {
  if (!currentTask.value) return;
  taskEditDescription.value = currentTask.value.description || '';
  taskEditCreateDate.value = currentTask.value.create_date || '';
  taskEditDoneDate.value = currentTask.value.done_date || '';
}

// 保存主任务详情
async function handleSaveTaskDetail() {
  if (!currentTask.value?.id) return;

  try {
    // 更新描述
    if (taskEditDescription.value !== (currentTask.value.description || '')) {
      await updateMainTaskDescription(currentTask.value.id, taskEditDescription.value);
    }

    // 更新创建日期
    if (taskEditCreateDate.value !== (currentTask.value.create_date || '')) {
      await updateMainTaskCreateDate(currentTask.value.id, taskEditCreateDate.value);
    }

    // 更新完成日期
    const currentDoneDate = currentTask.value.done_date || '';
    if (taskEditDoneDate.value !== currentDoneDate) {
      await updateMainTaskDoneDate(currentTask.value.id, taskEditDoneDate.value || null);
    }

    // 如果完成日期非空，自动设置为 done 状态
    if (taskEditDoneDate.value && !currentTask.value.is_done) {
      await toggleMainTaskStatus(currentTask.value.id, true);
    }

    // 重新加载任务数据
    tasks.value = await loadMainTasks();
    const updatedTask = tasks.value.find(t => t.id === currentTask.value?.id);
    if (updatedTask) {
      currentTask.value = updatedTask;
    }
    await notifyMainToRefresh();
  } catch (error) {
    console.error('Failed to save task detail:', error);
  }
}

// 保存主任务详情并返回列表
async function handleSaveTaskDetailAndBack() {
  await handleSaveTaskDetail();
  handleBackToList();
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

onMounted(async () => {
  loadSettings();

  // 加载所有主任务（只加载一次）
  tasks.value = await loadMainTasks();

  // 检查是否有初始化的 task_id
  const initialTaskId = (window as any).__TASK_ID__;
  if (initialTaskId) {
    const task = tasks.value.find(t => t.id === initialTaskId);
    if (task) {
      currentTask.value = task;
      initTaskEditData();
      await loadSubTasks();
    }
  }

  // 监听来自主窗口的 task_id
  const unlisten = await getCurrentWindow().listen<number>('set_task_id', async (event) => {
    const taskId = event.payload;
    // 重新加载任务列表以获取最新数据
    tasks.value = await loadMainTasks();
    const task = tasks.value.find(t => t.id === taskId);
    if (task) {
      currentTask.value = task;
      initTaskEditData();
      // 默认显示子任务列表视图
      viewMode.value = 'list';
      await loadSubTasks();
    }
  });

  window.addEventListener('storage', handleSettingsUpdate);

  await nextTick();
  requestAnimationFrame(async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
  });

  onUnmounted(() => {
    unlisten();
  });
});

onUnmounted(() => {
  window.removeEventListener('storage', handleSettingsUpdate);
});
</script>

<template>
  <div class="task-overlay fixed inset-0 z-[9999] flex w-full h-full" :style="themeStyle" :class="{ 'dark': effectiveTheme === 'dark' }">
    <div class="task-panel relative flex flex-col overflow-hidden w-full h-full rounded-lg transition-colors shadow-lg"
      :style="{
        backgroundColor: 'var(--theme-bg)',
        border: '1px solid var(--theme-border)',
        backdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
        WebkitBackdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
      }">

      <div class="title-bar flex items-center justify-between px-3 py-2.5 shrink-0 select-none group"
        data-tauri-drag-region>
        <!-- 主任务详情视图 -->
        <template v-if="viewMode === 'task'">
          <!-- 左侧返回按钮 -->
          <div class="w-[60px] flex items-center justify-start">
            <button @click="handleBackToList"
              class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
              :style="{ color: 'var(--theme-text)' }">
              <ChevronLeft class="w-4 h-4" />
            </button>
          </div>

          <!-- 中间标题，严格居中 -->
          <div class="flex-1 flex justify-center items-center h-6 px-2"
            @mousedown="(e) => e.target === e.currentTarget && handleIconDrag()">
            <template v-if="!isEditingTitle">
              <span
                class="text-base font-medium leading-relaxed transition-opacity truncate max-w-[200px] hover:opacity-80"
                :style="{ color: 'var(--theme-text)' }"
                @click="startEditingTitle(currentTask?.content || '')">
                {{ currentTask?.content || 'Task' }}
              </span>
            </template>
            <template v-else>
              <input
                ref="titleInputRef"
                v-model="editingTitle"
                type="text"
                class="w-full max-w-[200px] outline-none px-2 py-0.5 rounded text-base font-medium leading-relaxed text-center bg-white dark:bg-neutral-800 border border-transparent focus:border-[var(--theme-border)] shadow-sm"
                :style="{ color: 'var(--theme-text)' }"
                @blur="saveTaskTitle"
                @keydown.enter="saveTaskTitle"
                @keydown.escape="cancelEditingTitle"
              />
            </template>
          </div>

          <!-- 右侧按钮 -->
          <div class="w-[60px] flex items-center justify-end">
            <button @click="handleClose"
              class="close-btn shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
              :style="{ color: 'var(--theme-text)' }">
              <X class="w-4 h-4" />
            </button>
          </div>
        </template>

        <!-- 子任务列表视图 -->
        <template v-else-if="viewMode === 'list'">
          <!-- 左侧按钮 -->
          <div class="w-[60px] flex items-center justify-start">
            <button @click="handleBackToTask"
              class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
              :style="{ color: 'var(--theme-text)' }">
              <ChevronLeft class="w-4 h-4" />
            </button>
          </div>

          <!-- 中间标题，严格居中，可编辑 -->
          <div class="flex-1 flex justify-center items-center h-6 px-2"
            @mousedown="(e) => e.target === e.currentTarget && handleIconDrag()">
            <template v-if="!isEditingTitle">
              <span
                class="text-base font-medium leading-relaxed transition-opacity truncate max-w-[200px] hover:opacity-80"
                :style="{ color: 'var(--theme-text)' }"
                @click="startEditingTitle(currentTask?.content || '')">
                {{ currentTask?.content || 'Task' }}
              </span>
            </template>
            <template v-else>
              <input
                ref="titleInputRef"
                v-model="editingTitle"
                type="text"
                class="w-full max-w-[200px] outline-none px-2 py-0.5 rounded text-base font-medium leading-relaxed text-center bg-white dark:bg-neutral-800 border border-transparent focus:border-[var(--theme-border)] shadow-sm"
                :style="{ color: 'var(--theme-text)' }"
                @blur="saveTaskTitle"
                @keydown.enter="saveTaskTitle"
                @keydown.escape="cancelEditingTitle"
              />
            </template>
          </div>

          <!-- 右侧按钮 -->
          <div class="w-[60px] flex items-center justify-end gap-1">
            <button @click="handleStartAdding"
              class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
              :style="{ color: 'var(--theme-text)' }">
              <Plus class="w-4 h-4" />
            </button>
            <button @click="handleClose"
              class="close-btn shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
              :style="{ color: 'var(--theme-text)' }">
              <X class="w-4 h-4" />
            </button>
          </div>
        </template>

        <!-- 子任务编辑视图 -->
        <template v-else-if="viewMode === 'detail'">
          <!-- 左侧按钮 -->
          <div class="w-[60px] flex items-center justify-start">
            <button @click="handleBackToList"
              class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
              :style="{ color: 'var(--theme-text)' }">
              <ChevronLeft class="w-4 h-4" />
            </button>
          </div>

          <!-- 中间标题，不可编辑 -->
          <div class="flex-1 flex justify-center items-center h-6 px-2"
            @mousedown="handleIconDrag">
            <span
              class="text-base font-medium leading-relaxed truncate max-w-[200px]"
              :style="{ color: 'var(--theme-text)' }">
              {{ editingSubTask?.content || '...' }}
            </span>
          </div>

          <!-- 右侧按钮 -->
          <div class="w-[60px] flex items-center justify-end gap-1">
            <button @click="handleDeleteSubTask(editingSubTask!.id!)"
              class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-red-50 dark:hover:bg-red-900/30"
              :style="{ color: 'var(--theme-text-muted)' }">
              <Trash2 class="w-4 h-4 hover:text-red-500 dark:hover:text-red-400 transition-colors" />
            </button>
            <button @click="handleClose"
              class="close-btn shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
              :style="{ color: 'var(--theme-text)' }">
              <X class="w-4 h-4" />
            </button>
          </div>
        </template>
      </div>

      <div class="flex-1 relative overflow-hidden">
        <Transition name="view-fade" mode="out-in">
          <!-- 主任务详情视图 -->
          <div v-if="viewMode === 'task'" key="task" class="absolute inset-0 flex flex-col w-full h-full">
            <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar p-3">
              <div class="h-full rounded-lg p-3"
                :style="{
                  backgroundColor: 'var(--theme-cell)',
                  border: '1px solid var(--theme-border)',
                }">
                <ScheduleEditor
                  ref="scheduleEditorRef"
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

          <!-- 子任务列表视图 -->
          <div v-else-if="viewMode === 'list'" key="list" class="absolute inset-0 flex flex-col w-full h-full">
            <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
              <div class="space-y-2">
                <!-- 主任务详情入口按钮 -->
                <div
                  class="group flex items-center justify-center px-3 py-2 rounded-lg transition-all"
                  :style="{
                    backgroundColor: 'var(--theme-cell)',
                    border: '1px solid var(--theme-border)',
                  }"
                  @click="handleViewTaskDetail"
                >
                  <ListTodo class="w-4 h-4 opacity-60 group-hover:opacity-100 transition-opacity" :style="{ color: 'var(--theme-text-muted)' }" />
                </div>

                <!-- 新增子任务 -->
                <ListItem
                  v-if="isAdding"
                  key="add-new-subtask"
                  is-add-mode
                  @add="handleAddSubTask"
                  @cancel="handleCancelAdd"
                  @click.stop
                />

                <!-- 子任务列表 -->
                <ListItem
                  v-for="subTask in subTasks"
                  :key="subTask.id"
                  :title="subTask.content"
                  :preview="subTask.description"
                  :date="subTask.create_date"
                  :is-done="subTask.is_done"
                  center-calendar
                  @update:title="(val) => handleUpdateSubTask(subTask, val)"
                  @update:date="(val) => handleUpdateSubTaskDate(subTask, val)"
                  @delete="handleDeleteSubTask(subTask.id!)"
                  @toggle-done="handleToggleDone(subTask)"
                  @click="handleSelectSubTask(subTask)"
                />
              </div>

              <div v-if="subTasks.length === 0 && !isAdding" class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity">
                <div class="p-4 rounded-full" :style="{ backgroundColor: 'var(--theme-cell)' }">
                  <ListTodo class="w-8 h-8 opacity-20" :style="{ color: 'var(--theme-text)' }" />
                </div>
              </div>
            </div>
          </div>

          <!-- 子任务编辑视图 -->
          <div v-else-if="viewMode === 'detail'" key="detail" class="absolute inset-0 flex flex-col w-full h-full">
            <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar p-3">
              <div class="h-full rounded-lg p-3"
                :style="{
                  backgroundColor: 'var(--theme-cell)',
                  border: '1px solid var(--theme-border)',
                }">
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
        </Transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 优雅的悬浮滚动条 */
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

/* 日期输入框样式 */
input[type="date"]::-webkit-calendar-picker-indicator {
  filter: var(--theme-text-muted);
  cursor: pointer;
}
</style>
