<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import dayjs from 'dayjs';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import CalendarHeader from './components/calendar/CalendarHeader.vue';
import CalendarGrid from './components/calendar/CalendarGrid.vue';
import ResizeHandles from './components/ui/ResizeHandles.vue';
import ToastContainer from './components/ui/ToastContainer.vue';
import DescriptionDialog from './components/dialogs/DescriptionDialog.vue';
import BatchTaskDialog from './components/dialogs/BatchTaskDialog.vue';
import { useDatabase } from './composables/useDatabase';
import { useSchedules } from './composables/useSchedules';
import { useSettings } from './composables/useSettings';
import { useScheduleUndo } from './composables/useScheduleUndo';
import { useFonts } from './composables/useFonts';
import { useContextMenu } from './composables/useContextMenu';
import { setWindowLocked, hideWindow, exitApp } from './utils/window';
import { exportDatabaseBackup, exportAsZip, importFromJson } from './utils/export';
import { useToast } from './composables/useToast';
import { hexToRgba, adjustBrightness } from './utils/color';
import type { ViewMode, Schedule, BatchTaskConfig } from './types';

const { initDatabase, exportAllData, importAndMergeData } = useDatabase();
const {
  schedules,
  currentDate,
  viewMode,
  refreshSchedules,
  switchViewMode,
  resetSchedule,
  updateScheduleLines,
  setCellColor,
  prevMonth,
  nextMonth,
  goToToday,
  selectDate,
  toggleScheduleStatus,
  saveSchedule,
  updateScheduleDescription,
  updateScheduleContent,
  updateScheduleDate,
  updateScheduleFatherTask,
  batchAddSchedules,
} = useSchedules();
const { currentSettings, initSettings, saveSettings, getSetting } = useSettings();
const { showSuccess, showError } = useToast();
const { loadFonts } = useFonts();
const { pushAction, handleToggleDone: toggleDoneWithUndo, handleMoveSchedule: moveScheduleWithUndo, handleUndo: undo, handleRedo: redo, canUndo, canRedo } = useScheduleUndo(
  schedules,
  toggleScheduleStatus,
  refreshSchedules,
  updateScheduleLines,
  saveSchedule,
  updateScheduleDate
);
const { contextMenu, contextMenuStyle, showContextMenu, hideContextMenu, getSelectedDate, getColorOptions } = useContextMenu();

const calendarKey = ref(0);
const showMenu = ref(false);
const showMiniCalendar = ref(false);
const showDescriptionDialog = ref(false);
const showBatchTaskDialog = ref(false);
const editingSchedule = ref<Schedule | null>(null);
const isLocked = ref(false);
const isBoardVisible = ref(false);
const isNoteVisible = ref(false);
const isSearchVisible = ref(false);

const hideWeekends = computed(() => getSetting('hide_weekends') ?? false);

// 动态主题样式 - 与 NoteWindow/TaskWindow 保持一致
const themeStyle = computed(() => {
  const s = currentSettings.value;
  if (!s) return {};
  const bgOpacity = s.bg_opacity / 100;
  const cellOpacity = s.cell_opacity / 100;
  return {
    '--theme-bg': hexToRgba(s.bg_color, bgOpacity),
    '--theme-cell': hexToRgba(s.cell_color, cellOpacity),
    '--theme-text': s.text_color,
    '--theme-text-secondary': adjustBrightness(s.text_color, 30),
    '--theme-text-muted': s.muted_text_color,
    '--theme-primary': s.primary_color,
    '--theme-primary-alpha': hexToRgba(s.primary_color, 0.15),
    '--theme-border': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.05)'),
    'font-family': s.font_family,
    'font-size': `${s.font_size}px`,
    'backgroundColor': hexToRgba(s.bg_color, bgOpacity),
    'backdropFilter': s.enable_blur ? 'blur(16px) saturate(180%)' : 'none',
    'WebkitBackdropFilter': s.enable_blur ? 'blur(16px) saturate(180%)' : 'none',
  };
});

function closeOverlays() {
  showMenu.value = false;
  showMiniCalendar.value = false;
  hideContextMenu();
}

function toggleMenu() {
  showMenu.value = !showMenu.value;
  if (showMenu.value) showMiniCalendar.value = false;
}

function toggleMiniCalendar() {
  showMiniCalendar.value = !showMiniCalendar.value;
  if (showMiniCalendar.value) showMenu.value = false;
}

async function openSettings() {
  showMenu.value = false;
  await invoke('open_settings_window');
}

async function hideMainWindow() {
  showMenu.value = false;
  await hideWindow();
}

async function quitApp() {
  showMenu.value = false;
  await exitApp();
}

// 导出备份（JSON 格式）
async function handleExportBackup() {
  showMenu.value = false;
  try {
    const data = await exportAllData();
    const success = await exportDatabaseBackup(data);
    if (success) {
      showSuccess('备份导出成功');
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : '导出失败，请重试';
    showError(message);
  }
}

// 导出压缩包（ZIP 格式，包含 CSV）
async function handleExportZip() {
  showMenu.value = false;
  try {
    const data = await exportAllData();
    const success = await exportAsZip(data);
    if (success) {
      showSuccess('压缩包导出成功');
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : '导出失败，请重试';
    showError(message);
  }
}

// 导入备份（JSON 格式）
async function handleImportBackup() {
  showMenu.value = false;
  try {
    const data = await importFromJson();
    if (!data) return; // 用户取消

    const stats = await importAndMergeData(data);
    await refreshSchedules();

    const messages = [];
    if (stats.schedules.inserted > 0 || stats.schedules.updated > 0) {
      messages.push(`日程: 新增 ${stats.schedules.inserted}, 更新 ${stats.schedules.updated}`);
    }
    if (stats.mainTasks.inserted > 0 || stats.mainTasks.updated > 0) {
      messages.push(`主任务: 新增 ${stats.mainTasks.inserted}, 更新 ${stats.mainTasks.updated}`);
    }
    if (stats.notes.inserted > 0 || stats.notes.updated > 0) {
      messages.push(`备忘录: 新增 ${stats.notes.inserted}, 更新 ${stats.notes.updated}`);
    }
    if (stats.cellMetadata.inserted > 0 || stats.cellMetadata.updated > 0) {
      messages.push(`颜色标记: 新增 ${stats.cellMetadata.inserted}, 更新 ${stats.cellMetadata.updated}`);
    }

    showSuccess(messages.length > 0 ? `导入成功\n${messages.join('\n')}` : '导入成功：无数据变化');
  } catch (error) {
    const message = error instanceof Error ? error.message : '导入失败，请重试';
    showError(message);
  }
}

// 同步功能（预留）
function handleSync() {
  showMenu.value = false;
  showError('同步功能开发中，敬请期待');
}

function handleReset(date: string, content: string | null) {
  if (isLocked.value) return;
  resetSchedule(date, content);
}

function handleUpdate(date: string, lines: { id?: number; text: string; done: boolean }[], viewMode?: ViewMode) {
  if (isLocked.value) return;

  const currentSchedules = schedules.value.get(date) || [];
  const previousLines = currentSchedules
    .filter(s => s.id !== -1 && s.content.trim() !== '')
    .map(s => ({ id: s.id, text: s.content.trim(), done: !!s.is_done }));

  pushAction({
    type: 'updateLines',
    data: { date, previousLines },
    timestamp: Date.now(),
  });

  updateScheduleLines(date, lines, viewMode === 'done');
}

function handleSelectDate(date: dayjs.Dayjs) {
  selectDate(date);
  showMiniCalendar.value = false;
}

function handleSwitchViewMode(mode: ViewMode) {
  switchViewMode(mode);
}

function handleNavigate(dateStr: string) {
  currentDate.value = dayjs(dateStr);
  refreshSchedules();
}

function handleContextMenu(event: MouseEvent, date: string) {
  if (isLocked.value) return;
  showContextMenu(event, date);
}

function selectColor(color: string) {
  setCellColor(getSelectedDate(), color);
  hideContextMenu();
}

async function handleToggleDone(schedule: Schedule) {
  await toggleDoneWithUndo(schedule);
}

function handleEditDescription(schedule: Schedule) {
  if (isLocked.value) return;
  editingSchedule.value = schedule;
  showDescriptionDialog.value = true;
}

async function handleDescriptionSave(scheduleId: number, content: string, description: string, createDate: string, doneDate: string, fatherTask: number | null) {
  // 更新标题（如果内容有变化）
  if (editingSchedule.value && content !== editingSchedule.value.content) {
    await updateScheduleContent(scheduleId, content);
  }
  await updateScheduleDescription(scheduleId, description);
  if (createDate && createDate !== editingSchedule.value?.create_date) {
    await updateScheduleDate(scheduleId, 'create_date', createDate);
  }
  if (doneDate && doneDate !== editingSchedule.value?.done_date) {
    await updateScheduleDate(scheduleId, 'done_date', doneDate);
  }
  await updateScheduleFatherTask(scheduleId, fatherTask);
  showDescriptionDialog.value = false;
  editingSchedule.value = null;
}

function handleDescriptionCancel() {
  showDescriptionDialog.value = false;
  editingSchedule.value = null;
}

async function handleScheduleDrop(targetDate: string, scheduleId: number, sourceDate: string, viewMode?: ViewMode) {
  // 如果拖到同一个日期，不做任何操作
  if (targetDate === sourceDate) return;

  try {
    // todo 模式：修改 create_date
    // done 模式：修改 done_date
    const field = viewMode === 'done' ? 'done_date' : 'create_date';
    await moveScheduleWithUndo(scheduleId, field, sourceDate, targetDate);
  } catch (error) {
    console.error('Failed to move schedule:', error);
  }
}

function handleOpenBatchTask() {
  if (isLocked.value) return;
  showMenu.value = false;
  showBatchTaskDialog.value = true;
}

async function handleBatchTaskConfirm(config: BatchTaskConfig) {
  await batchAddSchedules(config);
  showBatchTaskDialog.value = false;
}

function handleBatchTaskCancel() {
  showBatchTaskDialog.value = false;
}

async function handleUndo() {
  await undo();
}

async function handleRedo() {
  await redo();
}

function toggleLock() {
  isLocked.value = !isLocked.value;
  setWindowLocked(isLocked.value);
}

async function handleSettingsUpdate() {
  await initSettings();
  calendarKey.value++;
  await refreshSchedules();
}

async function loadBoardVisibility() {
  const saved = localStorage.getItem('chronos_board_visible');
  if (saved === 'true') {
    isBoardVisible.value = true;
    await invoke('open_board_window');
  }
}

async function toggleBoard() {
  const isVisible = await invoke<boolean>('toggle_board_window');
  isBoardVisible.value = isVisible;
  localStorage.setItem('chronos_board_visible', String(isVisible));
}

async function loadNoteVisibility() {
  const saved = localStorage.getItem('chronos_note_visible');
  if (saved === 'true') {
    isNoteVisible.value = true;
    await invoke('open_note_window');
  }
}

async function toggleNote() {
  const isVisible = await invoke<boolean>('toggle_note_window');
  isNoteVisible.value = isVisible;
  localStorage.setItem('chronos_note_visible', String(isVisible));
}

async function loadSearchVisibility() {
  const saved = localStorage.getItem('chronos_search_visible');
  if (saved === 'true') {
    isSearchVisible.value = true;
    await invoke('open_search_window');
  }
}

async function toggleSearch() {
  const isVisible = await invoke<boolean>('toggle_search_window');
  isSearchVisible.value = isVisible;
  localStorage.setItem('chronos_search_visible', String(isVisible));
}

// 处理从搜索窗口导航到指定日期
function handleNavigateToDate(data: { date: string; viewMode: ViewMode }) {
  if (data.date) {
    currentDate.value = dayjs(data.date);
    if (data.viewMode) {
      switchViewMode(data.viewMode);
    }
    refreshSchedules();
  }
}

function toggleWeekends() {
  showMenu.value = false;
  const currentHideWeekends = getSetting('hide_weekends') ?? false;
  if (currentSettings.value) {
    saveSettings({ ...currentSettings.value, hide_weekends: !currentHideWeekends });
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (isLocked.value) return;
  if (event.key === 'PageUp') {
    event.preventDefault();
    prevMonth();
  } else if (event.key === 'PageDown') {
    event.preventDefault();
    nextMonth();
  }
}

onMounted(async () => {
  await initDatabase();
  await refreshSchedules();
  await initSettings();
  await loadFonts();
  await loadBoardVisibility();
  await loadNoteVisibility();
  await loadSearchVisibility();
  window.addEventListener('storage', handleSettingsUpdate);
  window.addEventListener('keydown', handleKeyDown);

  // 监听来自 TaskWindow 的数据变更事件
  listen('schedule-changed', async () => {
    await refreshSchedules();
  });

  // 监听来自 SearchWindow 的导航事件
  listen<{ date: string; viewMode: ViewMode }>('navigate-to-date', (event) => {
    handleNavigateToDate(event.payload);
  });
});

onUnmounted(() => {
  window.removeEventListener('storage', handleSettingsUpdate);
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <div
    class="glass rounded-lg overflow-hidden flex flex-col h-screen"
    :style="{
      borderWidth: 'var(--cell-border-width)',
      borderStyle: 'solid',
      borderColor: 'var(--cell-border-color)',
      ...themeStyle,
    }"
    @contextmenu.prevent
  >
    <ResizeHandles :is-locked="isLocked" />

    <CalendarHeader
      :current-date="currentDate"
      :show-mini-calendar="showMiniCalendar"
      :show-menu="showMenu"
      :is-locked="isLocked"
      :can-undo="canUndo()"
      :can-redo="canRedo()"
      :view-mode="viewMode"
      :is-board-visible="isBoardVisible"
      :is-note-visible="isNoteVisible"
      :is-search-visible="isSearchVisible"
      :hide-weekends="hideWeekends"
      @prev-month="prevMonth()"
      @next-month="nextMonth()"
      @go-today="goToToday()"
      @toggle-mini-calendar="toggleMiniCalendar"
      @toggle-menu="toggleMenu"
      @select-date="handleSelectDate"
      @settings="openSettings"
      @hide="hideMainWindow"
      @quit="quitApp"
      @export-backup="handleExportBackup"
      @export-zip="handleExportZip"
      @import-backup="handleImportBackup"
      @sync="handleSync"
      @toggle-lock="toggleLock"
      @undo="handleUndo"
      @redo="handleRedo"
      @switch-view-mode="handleSwitchViewMode"
      @open-batch-task="handleOpenBatchTask"
      @toggle-board="toggleBoard"
      @toggle-note="toggleNote"
      @toggle-search="toggleSearch"
      @toggle-weekends="toggleWeekends"
    />

    <CalendarGrid
      :key="calendarKey"
      :current-date="currentDate"
      :schedules="schedules"
      :is-locked="isLocked"
      :view-mode="viewMode"
      @reset="handleReset"
      @update="handleUpdate"
      @navigate="handleNavigate"
      @contextmenu="handleContextMenu"
      @toggle-done="handleToggleDone"
      @edit-description="handleEditDescription"
      @schedule-drop="handleScheduleDrop"
    />

    <div
      v-if="showMenu || showMiniCalendar || contextMenu.show"
      class="fixed inset-0 z-40"
      @mousedown.stop
      @click="closeOverlays"
    ></div>

    <Teleport to="body">
      <div
        v-if="contextMenu.show"
        class="fixed z-50 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1"
        :style="contextMenuStyle"
      >
        <button
          v-for="color in getColorOptions()"
          :key="color.value"
          @click="selectColor(color.value)"
          class="w-full px-3 py-1.5 text-center text-sm hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center justify-center gap-2"
        >
          <span
            v-if="color.value"
            class="w-4 h-4 rounded border border-gray-200 dark:border-gray-600"
            :style="{ backgroundColor: color.value }"
          ></span>
          <span v-else class="w-4 h-4 rounded border border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-700"></span>
          <span class="text-gray-700 dark:text-gray-300">{{ color.name }}</span>
        </button>
      </div>
    </Teleport>

    <DescriptionDialog
      :visible="showDescriptionDialog"
      :schedule="editingSchedule"
      @save="handleDescriptionSave"
      @cancel="handleDescriptionCancel"
    />

    <BatchTaskDialog
      :visible="showBatchTaskDialog"
      @confirm="handleBatchTaskConfirm"
      @cancel="handleBatchTaskCancel"
    />

    <ToastContainer />
  </div>
</template>
