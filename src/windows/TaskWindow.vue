<script setup lang="ts">
import { ref, onMounted, nextTick, computed, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { ListTodo } from 'lucide-vue-next';
import ListItem from '../components/ListItem.vue';
import ScheduleEditor from '../components/ScheduleEditor.vue';
import TaskWindowTitleBar from './TaskWindowTitleBar.vue';
import { useThemeStyle } from '../composables/useTaskTheme';
import { useTaskWindow } from '../composables/useTaskWindow';
import type { AppSettings } from '../types';
import { defaultLightSettings, defaultDarkSettings } from '../types';

const settings = ref<AppSettings>({ ...defaultLightSettings });

const effectiveTheme = computed(() => {
  if (settings.value.theme_mode === 'system') {
    return document.documentElement.getAttribute('data-theme') as 'light' | 'dark' || 'light';
  }
  return settings.value.theme_mode;
});

const { themeStyle, cellStyle } = useThemeStyle(settings, () => effectiveTheme.value);

const taskWindow = useTaskWindow();
const { currentTask, subTasks } = taskWindow;

const viewMode = ref<'task' | 'list' | 'detail'>('list');
const isAdding = ref(false);
const editingSubTask = ref<import('../types').Schedule | null>(null);

const isEditingTitle = ref(false);
const editingTitle = ref('');
const titleInputRef = ref<HTMLInputElement | null>(null);

const editDescription = ref('');
const editCreateDate = ref('');
const editDoneDate = ref('');
const scheduleEditorRef = ref<InstanceType<typeof ScheduleEditor> | null>(null);

const taskEditDescription = ref('');
const taskEditCreateDate = ref('');
const taskEditDoneDate = ref('');

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

function handleStartAdding() {
  isAdding.value = true;
}

function handleCancelAdd() {
  isAdding.value = false;
}

function handleViewTaskDetail() {
  isEditingTitle.value = false;
  viewMode.value = 'task';
}

function startEditingTitle(defaultTitle: string) {
  editingTitle.value = defaultTitle;
  isEditingTitle.value = true;
  nextTick(() => {
    titleInputRef.value?.focus();
    titleInputRef.value?.select();
  });
}

async function saveTaskTitle() {
  if (!currentTask.value?.id) return;
  const trimmed = editingTitle.value.trim();
  if (trimmed && trimmed !== currentTask.value.content) {
    await taskWindow.updateTaskTitle(currentTask.value.id, trimmed);
    currentTask.value.content = trimmed;
  }
  isEditingTitle.value = false;
}

function cancelEditingTitle() {
  isEditingTitle.value = false;
  editingTitle.value = '';
}

function handleBackToTask() {
  isEditingTitle.value = false;
  viewMode.value = 'task';
}

function handleSelectSubTask(subTask: import('../types').Schedule) {
  isEditingTitle.value = false;
  editingSubTask.value = subTask;
  editDescription.value = subTask.description || '';
  editCreateDate.value = subTask.create_date || '';
  editDoneDate.value = subTask.done_date || '';
  viewMode.value = 'detail';
  nextTick(() => scheduleEditorRef.value?.loadTasks());
}

function handleBackToList() {
  isEditingTitle.value = false;
  viewMode.value = 'list';
  setTimeout(() => {
    editingSubTask.value = null;
  }, 150);
}

async function handleSaveDetail() {
  if (!editingSubTask.value) return;
  await taskWindow.saveSubTaskDetail(editingSubTask.value, editDescription.value, editCreateDate.value, editDoneDate.value);
  handleBackToList();
}

function initTaskEditData() {
  if (!currentTask.value) return;
  taskEditDescription.value = currentTask.value.description || '';
  taskEditCreateDate.value = currentTask.value.create_date || '';
  taskEditDoneDate.value = currentTask.value.done_date || '';
}

async function handleSaveTaskDetail() {
  if (!currentTask.value?.id) return;
  await taskWindow.saveMainTaskDetail(
    currentTask.value.id,
    taskEditDescription.value,
    taskEditCreateDate.value,
    taskEditDoneDate.value,
    currentTask.value
  );
}

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

let unlisten: (() => void) | undefined;

onMounted(async () => {
  loadSettings();

  // 注册事件监听
  unlisten = await getCurrentWindow().listen<number>('set_task_id', async (event) => {
    await taskWindow.selectTask(event.payload);
    initTaskEditData();
    viewMode.value = 'list';
  });

  window.addEventListener('storage', handleSettingsUpdate);

  // 加载任务
  taskWindow.tasks.value = await taskWindow.loadMainTasks();

  const initialTaskId = (window as any).__TASK_ID__;
  if (initialTaskId) {
    const task = taskWindow.tasks.value.find(t => t.id === initialTaskId);
    if (task) {
      currentTask.value = task;
      initTaskEditData();
      await taskWindow.loadSubTasks();
    }
  }

  await nextTick();
  requestAnimationFrame(async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
  });
});

onUnmounted(() => {
  unlisten?.();
  window.removeEventListener('storage', handleSettingsUpdate);
});
</script>

<template>
  <div
    class="task-overlay fixed inset-0 z-[9999] flex w-full h-full"
    :style="themeStyle"
    :class="{ 'dark': effectiveTheme === 'dark' }"
  >
    <div
      class="task-panel relative flex flex-col overflow-hidden w-full h-full rounded-lg transition-colors shadow-lg"
      :style="{
        backgroundColor: 'var(--theme-bg)',
        border: '1px solid var(--theme-border)',
        backdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
        WebkitBackdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
      }"
    >
      <TaskWindowTitleBar
        :view-mode="viewMode"
        :is-editing-title="isEditingTitle"
        :editing-title="editingTitle"
        :current-title="viewMode === 'detail' ? (editingSubTask?.content || '...') : (currentTask?.content || 'Task')"
        :is-adding="isAdding"
        :theme-style="themeStyle"
        :title-input-ref="titleInputRef"
        @back="viewMode === 'task' ? handleBackToList() : viewMode === 'list' ? handleBackToTask() : handleBackToList()"
        @close="handleClose"
        @start-adding="handleStartAdding"
        @start-editing-title="startEditingTitle"
        @save-title="saveTaskTitle"
        @cancel-editing-title="cancelEditingTitle"
        @update:editing-title="editingTitle = $event"
        @start-drag="handleIconDrag"
      />

      <div class="flex-1 relative overflow-hidden">
        <Transition name="view-fade" mode="out-in">
          <!-- Main task detail view -->
          <div v-if="viewMode === 'task'" key="task" class="absolute inset-0 flex flex-col w-full h-full">
            <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar p-3">
              <div class="h-full rounded-lg p-3" :style="cellStyle">
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

          <!-- Subtask list view -->
          <div v-else-if="viewMode === 'list'" key="list" class="absolute inset-0 flex flex-col w-full h-full">
            <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
              <div class="space-y-2">
                <!-- Main task entry button -->
                <div
                  class="group flex items-center justify-center px-3 py-2 rounded-lg transition-all cursor-pointer"
                  :style="cellStyle"
                  @click="handleViewTaskDetail"
                >
                  <ListTodo class="w-4 h-4 opacity-60 group-hover:opacity-100 transition-opacity" :style="{ color: themeStyle['--theme-text-muted'] }" />
                </div>

                <!-- Add new subtask -->
                <ListItem
                  v-if="isAdding"
                  key="add-new-subtask"
                  is-add-mode
                  @add="(content) => { taskWindow.addSubTask(content); isAdding = false; }"
                  @cancel="handleCancelAdd"
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
                  @update:title="(val) => taskWindow.updateSubTaskContent(subTask, val)"
                  @update:date="(val) => taskWindow.updateSubTaskDate(subTask, val)"
                  @delete="taskWindow.removeSubTask(subTask.id!)"
                  @toggle-done="taskWindow.toggleSubTaskDone(subTask)"
                  @click="handleSelectSubTask(subTask)"
                />
              </div>

              <div v-if="subTasks.length === 0 && !isAdding" class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity">
                <div class="p-4 rounded-full" :style="cellStyle">
                  <ListTodo class="w-8 h-8 opacity-20" :style="{ color: themeStyle['--theme-text'] }" />
                </div>
              </div>
            </div>
          </div>

          <!-- Subtask detail view -->
          <div v-else-if="viewMode === 'detail'" key="detail" class="absolute inset-0 flex flex-col w-full h-full">
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
        </Transition>
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
