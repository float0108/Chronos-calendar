<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import dayjs from 'dayjs';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import CalendarHeader from './components/CalendarHeader.vue';
import CalendarGrid from './components/CalendarGrid.vue';
import ResizeHandles from './components/ResizeHandles.vue';
import ToastContainer from './components/ToastContainer.vue';
import ImportDialog from './components/ImportDialog.vue';
import DescriptionDialog from './components/DescriptionDialog.vue';
import BatchTaskDialog from './components/BatchTaskDialog.vue';
import { useDatabase } from './composables/useDatabase';
import { useSchedules } from './composables/useSchedules';
import { useSettings } from './composables/useSettings';
import { useImport } from './composables/useImport';
import { useScheduleUndo } from './composables/useScheduleUndo';
import { useFonts } from './composables/useFonts';
import { useContextMenu } from './composables/useContextMenu';
import { closeWindow, setWindowLocked } from './utils/window';
import type { ViewMode, Schedule, BatchTaskConfig } from './types';

const { initDatabase } = useDatabase();
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
  exportAllSchedules,
  importSchedulesFromData,
  toggleScheduleStatus,
  saveSchedule,
  updateScheduleDescription,
  updateScheduleContent,
  updateScheduleDate,
  updateScheduleFatherTask,
  batchAddSchedules,
} = useSchedules();
const { currentSettings, initSettings } = useSettings();
const { showImportDialog, pendingImportRecordCount, handleImport: handleImportData, performImport: performImportData, cancelImport } = useImport();
const { loadFonts } = useFonts();
const { pushAction, handleToggleDone: toggleDoneWithUndo, handleUndo: undo, handleRedo: redo, canUndo, canRedo } = useScheduleUndo(
  schedules,
  toggleScheduleStatus,
  refreshSchedules,
  updateScheduleLines,
  saveSchedule
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

async function quitApp() {
  await closeWindow();
}

async function handleExport() {
  showMenu.value = false;
  await exportAllSchedules();
}

async function handleImport() {
  showMenu.value = false;
  await handleImportData(importSchedulesFromData);
}

async function handleImportMerge() {
  await performImportData('merge', importSchedulesFromData);
}

async function handleImportOverwrite() {
  await performImportData('overwrite', importSchedulesFromData);
}

function handleImportCancel() {
  cancelImport();
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

function handleSelectDate(day: number) {
  selectDate(day);
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

async function handleDescriptionSave(scheduleId: number, content: string, description: string, dateField: 'create_date' | 'done_date' | null, dateValue: string | null, fatherTask: number | null, markDone: boolean) {
  // 更新标题（如果内容有变化）
  if (editingSchedule.value && content !== editingSchedule.value.content) {
    await updateScheduleContent(scheduleId, content);
  }
  await updateScheduleDescription(scheduleId, description);
  if (dateField && dateValue) {
    await updateScheduleDate(scheduleId, dateField, dateValue);
  }
  await updateScheduleFatherTask(scheduleId, fatherTask);
  if (markDone) {
    await toggleScheduleStatus(scheduleId, true);
  }
  showDescriptionDialog.value = false;
  editingSchedule.value = null;
}

function handleDescriptionCancel() {
  showDescriptionDialog.value = false;
  editingSchedule.value = null;
}

function handleOpenBatchTask() {
  if (isLocked.value) return;
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
  window.addEventListener('storage', handleSettingsUpdate);
  window.addEventListener('keydown', handleKeyDown);

  // 监听来自 TaskWindow 的数据变更事件
  listen('schedule-changed', async () => {
    await refreshSchedules();
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
      background: currentSettings?.enable_blur ? 'var(--glass-bg)' : 'var(--cell-bg)',
      backdropFilter: currentSettings?.enable_blur ? 'blur(var(--backdrop-blur)) saturate(var(--backdrop-saturate))' : 'none',
      WebkitBackdropFilter: currentSettings?.enable_blur ? 'blur(var(--backdrop-blur)) saturate(var(--backdrop-saturate))' : 'none',
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
      @prev-month="prevMonth()"
      @next-month="nextMonth()"
      @go-today="goToToday()"
      @toggle-mini-calendar="toggleMiniCalendar"
      @toggle-menu="toggleMenu"
      @select-date="handleSelectDate"
      @settings="openSettings"
      @quit="quitApp"
      @export="handleExport"
      @import="handleImport"
      @toggle-lock="toggleLock"
      @undo="handleUndo"
      @redo="handleRedo"
      @switch-view-mode="handleSwitchViewMode"
      @open-batch-task="handleOpenBatchTask"
      @toggle-board="toggleBoard"
      @toggle-note="toggleNote"
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
    />

    <div
      v-if="showMenu || showMiniCalendar || contextMenu.show"
      class="fixed inset-0 z-40"
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

    <ImportDialog
      :visible="showImportDialog"
      :record-count="pendingImportRecordCount"
      @merge="handleImportMerge"
      @overwrite="handleImportOverwrite"
      @cancel="handleImportCancel"
    />

    <DescriptionDialog
      :visible="showDescriptionDialog"
      :schedule="editingSchedule"
      :view-mode="viewMode"
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
