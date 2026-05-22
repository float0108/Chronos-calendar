<!-- components/SubTaskPanel.vue -->
<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { ListTodo, Info, Plus, ChevronLeft } from 'lucide-vue-next';
import WindowTitleBar from './WindowTitleBar.vue';
import ListItemPanel from './ListItemPanel.vue';
import ScheduleEditor from './ScheduleEditor.vue';
import type { MainTask, Schedule } from '../api/database';

type ViewMode = 'empty' | 'main-list' | 'main-detail' | 'sub-detail';

interface Props {
  currentTask: MainTask | null;
  subTasks: Schedule[];
  viewMode: ViewMode;
  editingSubTask: Schedule | null;
  themeStyle: Record<string, string>;
  cellStyle: Record<string, string>;
}

interface Emits {
  'select-sub-task': [subTask: Schedule];
  'toggle-sub-task-done': [subTask: Schedule];
  'view-task-detail': [];
  'back-to-list': [];
  'select-root-task': [];
  'sub-tasks-changed': [];
  'add-sub-task': [content: string];
  'delete-sub-task': [subTaskId: number];
  'update-sub-task-content': [subTaskId: number, content: string];
  'update-sub-task-date': [subTaskId: number, date: string];
  'update-sub-task-description': [subTaskId: number, description: string];
  'update-sub-task-done-date': [subTaskId: number, doneDate: string, isDone: boolean];
  'update-main-task-content': [taskId: number, content: string];
  'update-main-task-date': [taskId: number, date: string];
  'update-main-task-description': [taskId: number, description: string];
  'update-main-task-done-date': [taskId: number, doneDate: string, isDone: boolean];
  'title-saved': [];
  'close': [];
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// 本地状态
const isAddingSubTask = ref(false);
const addingKey = ref(0);
const editDescription = ref('');
const editCreateDate = ref('');
const editDoneDate = ref('');
const editIsDone = ref(false);
const taskEditDescription = ref('');
const taskEditCreateDate = ref('');
const taskEditDoneDate = ref('');
const taskEditIsDone = ref(false);
const subTaskSearchKeyword = ref('');

const scheduleEditorRef = ref<InstanceType<typeof ScheduleEditor> | null>(null);

// 是否显示搜索框
const showSearch = computed(() => !!(props.currentTask && props.viewMode === 'main-list'));

// 是否显示可编辑标题（sub-detail 和 main-detail）
const showEditableTitle = computed(() => props.viewMode === 'sub-detail' || props.viewMode === 'main-detail');

// 当前标题
const currentTitle = computed(() => {
  if (props.viewMode === 'sub-detail') {
    return props.editingSubTask?.content || '';
  }
  return props.currentTask?.content || '';
});

// 当 editingSubTask 变化时，更新编辑字段
watch(() => props.editingSubTask, (subTask) => {
  if (subTask) {
    editDescription.value = subTask.description || '';
    editCreateDate.value = subTask.create_date || '';
    editDoneDate.value = subTask.done_date || '';
    editIsDone.value = !!subTask.done_date;
  } else {
    editDescription.value = '';
    editCreateDate.value = '';
    editDoneDate.value = '';
    editIsDone.value = false;
  }
}, { deep: false });

// 当 currentTask 或 viewMode 变化时，更新 task 详情编辑字段
watch([() => props.currentTask, () => props.viewMode], ([task, mode]) => {
  if (task && mode === 'main-detail') {
    taskEditDescription.value = task.description || '';
    taskEditCreateDate.value = task.create_date || '';
    taskEditDoneDate.value = task.done_date || '';
    taskEditIsDone.value = !!task.done_date;
  }
}, { deep: false });

// ============ 导航 ============
const filteredSubTasks = computed(() => {
  if (!subTaskSearchKeyword.value.trim()) {
    return props.subTasks;
  }
  const keyword = subTaskSearchKeyword.value.toLowerCase();
  return props.subTasks.filter(task =>
    task.content.toLowerCase().includes(keyword) ||
    (task.description && task.description.toLowerCase().includes(keyword))
  );
});

// ============ 导航 ============

function handleBackToList() {
  emit('back-to-list');
  resetEditState();
}

function handleSelectRootTask() {
  if (props.viewMode === 'main-list') {
    emit('view-task-detail');
  } else {
    emit('select-root-task');
  }
}

function resetEditState() {
  editDescription.value = '';
  editCreateDate.value = '';
  editDoneDate.value = '';
  taskEditDescription.value = '';
  taskEditCreateDate.value = '';
  taskEditDoneDate.value = '';
  isAddingSubTask.value = false;
  addingKey.value = 0;
}

// ============ 子任务操作 ============

function handleStartAddSubTask() {
  isAddingSubTask.value = true;
}

function handleAddSubTask(content: string) {
  const trimmed = content.trim();
  if (!trimmed) {
    isAddingSubTask.value = false;
    addingKey.value = 0;
    return;
  }
  emit('add-sub-task', trimmed);
  isAddingSubTask.value = false;
  addingKey.value = 0;
}

function handleSelectSubTask(subTask: Schedule) {
  emit('select-sub-task', subTask);
}

function handleDeleteSubTask(subTaskId: number) {
  emit('delete-sub-task', subTaskId);
}

function handleToggleSubTaskDone(subTask: Schedule) {
  emit('toggle-sub-task-done', subTask);
}

function handleTitleUpdate(newTitle: string) {
  if (props.viewMode === 'sub-detail' && props.editingSubTask?.id) {
    emit('update-sub-task-content', props.editingSubTask.id, newTitle);
  } else if (props.viewMode === 'main-detail' && props.currentTask?.id) {
    emit('update-main-task-content', props.currentTask.id, newTitle);
  }
}

function handleTitleSaved() {
  emit('title-saved');
}

function handleUpdateSubTaskTitle(subTask: Schedule, newTitle: string) {
  if (subTask.id) {
    emit('update-sub-task-content', subTask.id, newTitle);
  }
}

function handleUpdateSubTaskDate(subTask: Schedule, newDate: string) {
  if (subTask.id) {
    emit('update-sub-task-date', subTask.id, newDate);
  }
}

function handleInfoClick() {
  emit('view-task-detail');
}

// ============ 详情编辑 ============

function handleSubTaskDoneDateChanged(_doneDate: string, isDone: boolean) {
  editIsDone.value = isDone;
}

function handleMainTaskDoneDateChanged(_doneDate: string, isDone: boolean) {
  taskEditIsDone.value = isDone;
}

function handleSaveDetail() {
  if (props.editingSubTask?.id) {
    emit('update-sub-task-description', props.editingSubTask.id, editDescription.value);
    emit('update-sub-task-done-date', props.editingSubTask.id, editDoneDate.value, editIsDone.value);
  }
  handleBackToList();
}

function handleSaveTaskDetailAndBack() {
  if (props.currentTask?.id) {
    emit('update-main-task-description', props.currentTask.id, taskEditDescription.value);
    emit('update-main-task-done-date', props.currentTask.id, taskEditDoneDate.value, taskEditIsDone.value);
  }
  handleBackToList();
}

function handleClose() {
  emit('close');
}
</script>

<template>
  <div class="right-panel flex flex-col flex-1 min-w-0">
    <!-- Header with WindowTitleBar -->
    <WindowTitleBar
      :theme-style="themeStyle"
      :show-search="showSearch"
      :editable-title="showEditableTitle"
      :title="currentTitle"
      v-model="subTaskSearchKeyword"
      search-placeholder="Search subtasks..."
      title-placeholder="Subtask title"
      @close="handleClose"
      @update:title="handleTitleUpdate"
      @title-saved="handleTitleSaved"
    >
      <template #left>
        <button
          v-if="currentTask"
          @click="handleSelectRootTask"
          class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
          :style="{ color: themeStyle['--theme-text'] }"
          :title="currentTask.content"
        >
          <ChevronLeft class="w-4 h-4" />
        </button>
      </template>

      <template #right>
        <button
          v-if="currentTask && viewMode === 'main-list'"
          @click="handleStartAddSubTask"
          class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
          :style="{ color: themeStyle['--theme-text'] }"
          title="Add subtask"
        >
          <Plus class="w-4 h-4" />
        </button>
      </template>
    </WindowTitleBar>

    <!-- Content Area -->
    <div class="flex-1 relative overflow-hidden">
      <!-- Subtask list view -->
      <div v-if="viewMode === 'main-list'" class="absolute inset-0 flex flex-col w-full h-full">
        <ListItemPanel
          :items="filteredSubTasks"
          :selected-item="null"
          :theme-style="themeStyle"
          :cell-style="cellStyle"
          :parent-id="currentTask?.id"
          :show-info-entry="true"
          :info-entry-icon="Info"
          :is-adding="isAddingSubTask"
          :show-header="false"
          @select="handleSelectSubTask"
          @add="handleAddSubTask"
          @delete="handleDeleteSubTask"
          show-context-menu-done
          @toggle-done="handleToggleSubTaskDone"
          @info-click="handleInfoClick"
          @start-add="handleStartAddSubTask"
          @update:is-adding="(_val) => isAddingSubTask = _val"
          @update-title="handleUpdateSubTaskTitle"
          @update-date="handleUpdateSubTaskDate"
        />
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
              @done-date-changed="handleSubTaskDoneDateChanged"
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
              @done-date-changed="handleMainTaskDoneDateChanged"
            />
          </div>
        </div>
      </div>

      <!-- Empty state: no task selected -->
      <div v-else class="absolute inset-0 flex flex-col items-center justify-center">
        <div class="p-4 rounded-full mb-4" :style="cellStyle">
          <ListTodo
            class="w-10 h-10 opacity-20"
            :style="{ color: themeStyle['--theme-text'] }"
          />
        </div>
        <p class="text-sm" :style="{ color: themeStyle['--theme-text-muted'] }">
          Select a task to view its schedules
        </p>
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
