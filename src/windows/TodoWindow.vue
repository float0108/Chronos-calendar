<script setup lang="ts">
import { ref, onMounted, nextTick, computed, onUnmounted, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { Plus, CheckSquare, Calendar, ChevronLeft } from 'lucide-vue-next';
import ListItem from '../components/ListItem.vue';
import WindowTitleBar from '../components/WindowTitleBar.vue';
import ScheduleEditor from '../components/ScheduleEditor.vue';
import { useThemeStyle } from '../composables/useTaskTheme';
import {
  loadTodoSchedules,
  searchSchedules,
  toggleScheduleStatus,
  updateScheduleContent,
  updateScheduleDate,
  updateScheduleDescription,
  deleteSchedule,
  type Schedule
} from '../api/database';
import type { AppSettings, DataChange } from '../types';
import { defaultLightSettings, defaultDarkSettings } from '../types';
import dayjs from 'dayjs';

const settings = ref<AppSettings>({ ...defaultLightSettings });
const schedules = ref<Schedule[]>([]);
const searchKeyword = ref('');

// 实际主题（解析 system）
const effectiveTheme = computed(() => {
  if (settings.value.theme_mode === 'system') {
    return document.documentElement.getAttribute('data-theme') as 'light' | 'dark' || 'light';
  }
  return settings.value.theme_mode;
});

const { themeStyle, cellStyle } = useThemeStyle(settings, () => effectiveTheme.value);

// 新增模式
const isAdding = ref(false);

// 是否显示过去的未完成日程
const showPastTodos = ref(false);

// 视图模式: list=列表, detail=详情
const viewMode = ref<'list' | 'detail'>('list');

// 当前编辑的日程
const currentSchedule = ref<Schedule | null>(null);
const editDescription = ref('');
const editCreateDate = ref('');
const editDoneDate = ref('');
const scheduleEditorRef = ref<InstanceType<typeof ScheduleEditor> | null>(null);

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
}

function handleSettingsUpdate() {
  loadSettings();
}

async function loadSchedulesData() {
  const today = dayjs();
  const endDate = today.add(1, 'year').format('YYYY-MM-DD');

  try {
    if (searchKeyword.value.trim()) {
      // 搜索时过滤未完成的
      const allResults = await searchSchedules(searchKeyword.value);
      schedules.value = allResults.filter(s => !s.is_done);
    } else {
      if (showPastTodos.value) {
        // 显示所有未完成的日程（过去+未来）
        const startDate = today.subtract(10, 'year').format('YYYY-MM-DD');
        schedules.value = await loadTodoSchedules(startDate, endDate);
      } else {
        // 只显示今天及以后的未完成日程
        const startDate = today.format('YYYY-MM-DD');
        schedules.value = await loadTodoSchedules(startDate, endDate);
      }
    }
  } catch (error) {
    console.error('Failed to load schedules:', error);
    schedules.value = [];
  }
}

function handleStartAdding() {
  isAdding.value = true;
}

async function handleAddSchedule(content: string) {
  const trimmed = content.trim();
  if (!trimmed) {
    isAdding.value = false;
    return;
  }

  try {
    const today = dayjs().format('YYYY-MM-DD');
    await import('../api/database').then(db => db.saveSchedule(today, trimmed, false));
    await loadSchedulesData();
  } catch (error) {
    console.error('Failed to add schedule:', error);
  }
  isAdding.value = false;
}

function handleCancelAdd() {
  isAdding.value = false;
}

async function handleToggleDone(schedule: Schedule) {
  if (!schedule.id) return;
  try {
    await toggleScheduleStatus(schedule.id, !schedule.is_done);
    await loadSchedulesData();
  } catch (error) {
    console.error('Failed to toggle schedule:', error);
  }
}

async function handleDeleteSchedule(scheduleId: number) {
  try {
    await deleteSchedule(scheduleId);
    await loadSchedulesData();
  } catch (error) {
    console.error('Failed to delete schedule:', error);
  }
}

async function handleUpdateSchedule(schedule: Schedule, newContent: string) {
  if (!schedule.id) return;
  const trimmed = newContent.trim();
  if (!trimmed) {
    await handleDeleteSchedule(schedule.id);
    return;
  }
  if (trimmed === schedule.content) return;
  try {
    await updateScheduleContent(schedule.id, trimmed);
    await loadSchedulesData();
  } catch (error) {
    console.error('Failed to update schedule:', error);
  }
}

async function handleUpdateScheduleDate(schedule: Schedule, newDate: string) {
  if (!schedule.id) return;
  try {
    await updateScheduleDate(schedule.id, 'create_date', newDate);
    await loadSchedulesData();
  } catch (error) {
    console.error('Failed to update schedule date:', error);
  }
}

function handleScheduleClick(schedule: Schedule) {
  currentSchedule.value = schedule;
  editDescription.value = schedule.description || '';
  editCreateDate.value = schedule.create_date || '';
  editDoneDate.value = schedule.done_date || '';
  viewMode.value = 'detail';
  nextTick(() => scheduleEditorRef.value?.loadTasks());
}

function handleBackToList() {
  viewMode.value = 'list';
  currentSchedule.value = null;
}

async function handleSaveScheduleDetail() {
  if (!currentSchedule.value?.id) return;
  try {
    if (editDescription.value !== (currentSchedule.value.description || '')) {
      await updateScheduleDescription(currentSchedule.value.id, editDescription.value || null);
    }
    if (editCreateDate.value !== (currentSchedule.value.create_date || '')) {
      await updateScheduleDate(currentSchedule.value.id, 'create_date', editCreateDate.value);
    }
    if (editDoneDate.value !== (currentSchedule.value.done_date || '')) {
      await updateScheduleDate(currentSchedule.value.id, 'done_date', editDoneDate.value);
    }
    await loadSchedulesData();
    handleBackToList();
  } catch (error) {
    console.error('Failed to save schedule detail:', error);
  }
}

function handleCancelScheduleDetail() {
  handleBackToList();
}

watch(showPastTodos, () => {
  loadSchedulesData();
});

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

  // 监听来自其他窗口的数据变更事件
  unlisten = await listen<DataChange>('schedule-changed', async (event) => {
    const change = event.payload;
    if (change.entity === 'schedule' || change.entity === 'batch') {
      await loadSchedulesData();
    }
  });

  window.addEventListener('storage', handleSettingsUpdate);

  await loadSchedulesData();
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
  <div class="todo-overlay fixed inset-0 z-[9999] flex w-full h-full" :style="themeStyle">
    <div class="todo-panel relative flex flex-col overflow-hidden w-full h-full rounded-lg transition-colors shadow-lg"
      :style="{
        backgroundColor: 'var(--theme-bg)',
        border: '1px solid var(--theme-border)',
        backdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
        WebkitBackdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
      }">

      <!-- 标题栏 - 使用单个 WindowTitleBar -->
      <WindowTitleBar
        :theme-style="themeStyle"
        :show-search="viewMode === 'list'"
        title="Todo"
        v-model="searchKeyword"
        @close="handleClose"
        @start-drag="handleIconDrag"
        @search="loadSchedulesData"
      >
        <template #left>
          <!-- 详情视图显示返回按钮 -->
          <button v-if="viewMode === 'detail'" @mousedown.stop="handleBackToList"
            class="shrink-0 w-6 h-6 flex items-center justify-center cursor-pointer hover:opacity-80 transition-opacity"
            :style="{ color: 'var(--theme-text)' }"
            title="返回">
            <ChevronLeft class="w-4 h-4" />
          </button>
          <!-- 列表视图显示拖拽按钮 -->
          <button v-else @mousedown.stop="handleIconDrag"
            class="shrink-0 w-6 h-6 flex items-center justify-center cursor-grab active:cursor-grabbing hover:opacity-80 transition-opacity"
            :style="{ color: 'var(--theme-text)' }"
            title="Drag">
            <CheckSquare class="w-4 h-4" />
          </button>
        </template>

        <!-- 详情视图标题 -->
        <template #center>
          <span v-if="viewMode === 'detail'" class="text-base font-medium leading-relaxed" :style="{ color: 'var(--theme-text)' }">
            日程详情
          </span>
        </template>

        <template #right>
          <button v-if="viewMode === 'list'" @click="showPastTodos = !showPastTodos"
            class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
            :style="{ color: showPastTodos ? 'var(--theme-primary)' : 'var(--theme-text)' }"
            title="显示过去的未完成日程">
            <Calendar class="w-4 h-4" />
          </button>
          <button v-if="viewMode === 'list'" @click="handleStartAdding"
            class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
            :style="{ color: 'var(--theme-text)' }">
            <Plus class="w-4 h-4" />
          </button>
        </template>
      </WindowTitleBar>

      <!-- 内容区域 -->
      <div class="flex-1 relative overflow-hidden">
        <Transition name="view-fade" mode="out-in">
          <!-- 列表视图 -->
          <div v-if="viewMode === 'list'" key="list" class="absolute inset-0 flex flex-col w-full h-full">
            <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
              <div class="space-y-2">
                <ListItem
                  v-if="isAdding"
                  key="add-new-schedule"
                  is-add-mode
                  @add="handleAddSchedule"
                  @cancel="handleCancelAdd"
                  @click.stop
                />

                <ListItem
                  v-for="schedule in schedules"
                  :key="schedule.id"
                  :title="schedule.content"
                  :preview="schedule.description"
                  :date="schedule.create_date"
                  :is-done="schedule.is_done"
                  center-calendar
                  @update:title="(val) => handleUpdateSchedule(schedule, val)"
                  @update:date="(val) => handleUpdateScheduleDate(schedule, val)"
                  @delete="handleDeleteSchedule(schedule.id!)"
                  @toggle-done="handleToggleDone(schedule)"
                  @click="handleScheduleClick(schedule)"
                />
              </div>

              <div v-if="schedules.length === 0 && !isAdding" class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity">
                <div class="p-4 rounded-full" :style="cellStyle">
                  <CheckSquare class="w-8 h-8 opacity-20" :style="{ color: 'var(--theme-text)' }" />
                </div>
              </div>
            </div>
          </div>

          <!-- 详情视图 -->
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
                  :show-father-task="false"
                  @save="handleSaveScheduleDetail"
                  @cancel="handleCancelScheduleDetail"
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
/* 视图切换动画 */
.view-fade-enter-active,
.view-fade-leave-active {
  transition: opacity 0.2s ease;
}

.view-fade-enter-from,
.view-fade-leave-to {
  opacity: 0;
}

/* 优雅的悬浮滚动条，默认隐藏，hover时显示 */
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