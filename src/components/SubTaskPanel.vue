<!-- components/SubTaskPanel.vue -->
<script setup lang="ts">
import { ref } from 'vue';
import { ListTodo, Info, Plus } from 'lucide-vue-next';
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
  'view-task-detail': [];
  'back-to-list': [];
  'select-root-task': [];
  'sub-tasks-changed': [];
  'add-sub-task': [content: string];
  'delete-sub-task': [subTaskId: number];
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
const taskEditDescription = ref('');
const taskEditCreateDate = ref('');
const taskEditDoneDate = ref('');

const scheduleEditorRef = ref<InstanceType<typeof ScheduleEditor> | null>(null);

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

function handleInfoClick() {
  emit('view-task-detail');
}

function handleClose() {
  emit('close');
}

// ============ 详情编辑 ============

function handleSaveDetail() {
  handleBackToList();
}

function handleSaveTaskDetailAndBack() {
  handleBackToList();
}
</script>

<template>
  <div class="right-panel flex flex-col flex-1 min-w-0">
    <!-- Header with WindowTitleBar -->
    <WindowTitleBar
      :theme-style="themeStyle"
      @close="handleClose"
    >
      <!-- Breadcrumb navigation -->
      <nav v-if="currentTask" class="flex items-center gap-1 text-sm min-w-0 flex-1">
        <!-- Task name -->
        <button
          @click="handleSelectRootTask"
          class="px-2 py-1 rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors shrink-0"
          :style="{ color: viewMode === 'main-list' ? themeStyle['--theme-text'] : themeStyle['--theme-text-muted'] }"
          :title="currentTask.content"
        >
          <span class="truncate max-w-[150px] inline-block align-bottom">{{ currentTask.content }}</span>
        </button>

        <!-- Task Info breadcrumb -->
        <template v-if="viewMode === 'main-detail'">
          <span class="text-[var(--theme-text-muted)] shrink-0">/</span>
          <span class="px-2 py-1 shrink-0" :style="{ color: themeStyle['--theme-text'] }">Info</span>
        </template>

        <!-- Sub-task breadcrumb -->
        <template v-if="viewMode === 'sub-detail' && editingSubTask">
          <span class="text-[var(--theme-text-muted)] shrink-0">/</span>
          <span
            class="px-2 py-1 truncate max-w-[150px] inline-block align-bottom"
            :style="{ color: themeStyle['--theme-text'] }"
          >
            {{ editingSubTask.content }}
          </span>
        </template>
      </nav>
      <span v-else class="text-base font-medium" :style="{ color: themeStyle['--theme-text-muted'] }">
        Select a task
      </span>

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
          :items="subTasks"
          :selected-item="null"
          :theme-style="themeStyle"
          :cell-style="cellStyle"
          :parent-id="currentTask?.id"
          :show-info-entry="true"
          :info-entry-icon="Info"
          :is-adding="isAddingSubTask"
          @select="handleSelectSubTask"
          @add="handleAddSubTask"
          @delete="handleDeleteSubTask"
          @info-click="handleInfoClick"
          @start-add="handleStartAddSubTask"
          @update:is-adding="(_val) => isAddingSubTask = _val"
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
