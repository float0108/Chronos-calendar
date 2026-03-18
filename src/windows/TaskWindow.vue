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
  updateScheduleContent,
  updateScheduleDescription,
  updateScheduleDate,
  loadMainTasks,
  type Schedule,
  type MainTask
} from '../composables/db';
import { initDatabase } from '../composables/db/connection';
import { hexToRgba, adjustBrightness } from '../utils/color';
import ScheduleEditor from '../components/ScheduleEditor.vue';
import type { AppSettings } from '../types';
import { defaultLightSettings, defaultDarkSettings } from '../types';

const settings = ref<AppSettings>({ ...defaultLightSettings });
const tasks = ref<MainTask[]>([]);
const currentTask = ref<MainTask | null>(null);
const subTasks = ref<Schedule[]>([]);

// 新增模式
const isAdding = ref(false);

// 视图控制：false 显示列表，true 显示详情
const isEditing = ref(false);
const editingSubTask = ref<Schedule | null>(null);

// 编辑器数据
const editDescription = ref('');
const editCreateDate = ref('');
const editDoneDate = ref('');
const scheduleEditorRef = ref<InstanceType<typeof ScheduleEditor> | null>(null);

// 动态主题样式
const themeStyle = computed(() => {
  const s = settings.value;
  const bgOpacity = s.bg_opacity / 100;
  const cellOpacity = s.cell_opacity / 100;
  return {
    '--theme-bg': hexToRgba(s.bg_color, bgOpacity),
    '--theme-cell': hexToRgba(s.cell_color, cellOpacity),
    '--theme-text': s.text_color,
    '--theme-text-secondary': adjustBrightness(s.text_color, 30),
    '--theme-text-muted': adjustBrightness(s.text_color, 50),
    '--theme-primary': s.primary_color,
    '--theme-primary-alpha': hexToRgba(s.primary_color, 0.2),
    '--theme-border': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.05)'),
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
    const defaults = parsed.theme_mode === 'dark' ? defaultDarkSettings : defaultLightSettings;
    settings.value = { ...defaults, ...parsed };
  }
  applyTheme();
}

function applyTheme() {
  const s = settings.value;
  const root = document.documentElement;
  root.style.setProperty('--primary', s.primary_color);
  root.style.setProperty('--text-primary', s.text_color);
  root.style.setProperty('--text-muted', adjustBrightness(s.text_color, 60));
  root.setAttribute('data-theme', s.theme_mode);
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

// 点击条目进入详情
function handleSelectSubTask(subTask: Schedule) {
  editingSubTask.value = subTask;
  editDescription.value = subTask.description || '';
  editCreateDate.value = subTask.create_date || '';
  editDoneDate.value = subTask.done_date || '';
  isEditing.value = true;
  // 确保任务列表已加载
  nextTick(() => scheduleEditorRef.value?.loadTasks());
}

// 返回列表
function handleBackToList() {
  isEditing.value = false;
  setTimeout(() => {
    editingSubTask.value = null;
  }, 300);
}

// 保存详情
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

    await loadSubTasks();
    await notifyMainToRefresh();
    handleBackToList();
  } catch (error) {
    console.error('Failed to save detail:', error);
  }
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
  await initDatabase();

  // 监听来自主窗口的 task_id
  const unlisten = await getCurrentWindow().listen<number>('set_task_id', async (event) => {
    const taskId = event.payload;
    // 重新加载任务列表以确保获取最新数据
    tasks.value = await loadMainTasks();
    const task = tasks.value.find(t => t.id === taskId);
    if (task) {
      currentTask.value = task;
      await loadSubTasks();
    }
  });

  // 加载所有主任务
  tasks.value = await loadMainTasks();

  // 检查是否有初始化的 task_id
  const taskId = (window as any).__TASK_ID__;
  if (taskId) {
    const task = tasks.value.find(t => t.id === taskId);
    if (task) {
      currentTask.value = task;
      await loadSubTasks();
    }
  }

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
  <div class="task-overlay fixed inset-0 z-[9999] flex w-full h-full" :style="themeStyle" :class="{ 'dark': settings.theme_mode === 'dark' }">
    <div class="task-panel relative flex flex-col overflow-hidden w-full h-full rounded-lg transition-colors shadow-lg"
      :style="{
        backgroundColor: 'var(--theme-bg)',
        border: '1px solid var(--theme-border)',
        backdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
        WebkitBackdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
      }">

      <div class="title-bar flex items-center gap-2 px-3 py-2.5 shrink-0 select-none group"
        data-tauri-drag-region>
        <!-- 列表视图 -->
        <template v-if="!isEditing">
          <button @mousedown="handleIconDrag"
            class="shrink-0 w-6 h-6 flex items-center justify-center cursor-grab active:cursor-grabbing hover:opacity-80 transition-opacity"
            :style="{ color: 'var(--theme-text)' }"
            title="Drag">
            <ListTodo class="w-4 h-4" />
          </button>

          <div class="flex-1 min-w-0 flex justify-center items-center relative h-6"
            @mousedown="(e) => e.target === e.currentTarget && handleIconDrag()">
            <span class="text-[14px] font-medium leading-relaxed transition-opacity truncate max-w-[180px]"
              :style="{ color: 'var(--theme-text)' }">
              {{ currentTask?.content || 'Task' }}
            </span>
          </div>

          <button @click="handleStartAdding"
            class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
            :style="{ color: 'var(--theme-text)' }">
            <Plus class="w-4 h-4" />
          </button>
        </template>

        <!-- 编辑视图 -->
        <template v-else>
          <button @click="handleBackToList"
            class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
            :style="{ color: 'var(--theme-text)' }">
            <ChevronLeft class="w-4 h-4" />
          </button>

          <div class="flex-1 min-w-0 flex justify-center items-center h-6"
            @mousedown="handleIconDrag">
            <span class="text-[14px] font-medium leading-relaxed truncate max-w-[180px]"
              :style="{ color: 'var(--theme-text)' }">
              {{ editingSubTask?.content || '...' }}
            </span>
          </div>

          <button @click="handleDeleteSubTask(editingSubTask!.id!)"
            class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-red-50 dark:hover:bg-red-900/30"
            :style="{ color: 'var(--theme-text-muted)' }">
            <Trash2 class="w-4 h-4 hover:text-red-500 dark:hover:text-red-400 transition-colors" />
          </button>
        </template>

        <button @click="handleClose"
          class="close-btn shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
          :style="{ color: 'var(--theme-text)' }">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="flex-1 relative overflow-hidden">
        <!-- 列表视图 -->
        <Transition name="view-slide">
          <div v-if="!isEditing" class="absolute inset-0 flex flex-col w-full h-full">
            <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
              <TransitionGroup name="list" tag="div" class="space-y-2">
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
              </TransitionGroup>

              <div v-if="subTasks.length === 0 && !isAdding" class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity">
                <div class="p-4 rounded-full" :style="{ backgroundColor: 'var(--theme-cell)' }">
                  <ListTodo class="w-8 h-8 opacity-20" :style="{ color: 'var(--theme-text)' }" />
                </div>
              </div>
            </div>
          </div>

          <!-- 编辑视图 -->
          <div v-else class="absolute inset-0 flex flex-col w-full h-full z-10">
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

/* 列表动画 */
.list-move,
.list-enter-active,
.list-leave-active {
  transition: all 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateY(4px) scale(0.99);
}

.list-leave-active {
  position: absolute;
  width: calc(100% - 24px);
}

/* 视图切换动画 */
.view-slide-enter-active,
.view-slide-leave-active {
  transition: all 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}

.view-slide-enter-from {
  opacity: 0;
  transform: translateX(16px) scale(0.99);
}

.view-slide-leave-to {
  opacity: 0;
  transform: translateX(-16px) scale(0.99);
}

/* 日期输入框样式 */
input[type="date"]::-webkit-calendar-picker-indicator {
  filter: var(--theme-text-muted);
  cursor: pointer;
}
</style>
