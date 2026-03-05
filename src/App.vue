<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import dayjs from 'dayjs';
import CalendarHeader from './components/CalendarHeader.vue';
import CalendarGrid from './components/CalendarGrid.vue';
import ResizeHandles from './components/ResizeHandles.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import ToastContainer from './components/ToastContainer.vue';
import ImportDialog from './components/ImportDialog.vue';
import DescriptionDialog from './components/DescriptionDialog.vue';
import { useDatabase } from './composables/useDatabase';
import { useSchedules } from './composables/useSchedules';
import { useSettings } from './composables/useSettings';
import { useImport } from './composables/useImport';
import { useScheduleUndo } from './composables/useScheduleUndo';
import { closeWindow, setWindowLocked } from './utils/window';
import { colorOptions } from './constants';
import type { AppSettings, ThemeMode, ViewMode, Schedule } from './types';
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
} = useSchedules();
const {
  currentSettings,
  currentMode,
  initSettings,
  saveSettingsForMode,
  switchMode,
} = useSettings();
const { showImportDialog, pendingImportRecordCount, handleImport: handleImportData, performImport: performImportData, cancelImport } = useImport();
const { pushAction, handleToggleDone: toggleDoneWithUndo, handleUndo: undo, handleRedo: redo, canUndo, canRedo } = useScheduleUndo(
  schedules,
  toggleScheduleStatus,
  refreshSchedules,
  updateScheduleLines,
  saveSchedule
);
const calendarKey = ref(0);
const showMenu = ref(false);
const showMiniCalendar = ref(false);
const showSettings = ref(false);
const showDescriptionDialog = ref(false);
const editingSchedule = ref<Schedule | null>(null);
const isLocked = ref(false);
const contextMenu = ref<{ show: boolean; x: number; y: number; date: string }>({
  show: false,
  x: 0,
  y: 0,
  date: '',
});
const contextMenuStyle = computed(() => {
  const menuWidth = 100;
  const menuHeight = 280;
  let x = contextMenu.value.x;
  let y = contextMenu.value.y;
  
  if (typeof window !== 'undefined') {
    if (x + menuWidth > window.innerWidth) {
      x = window.innerWidth - menuWidth - 10;
    }
    if (y + menuHeight > window.innerHeight) {
      y = window.innerHeight - menuHeight - 10;
    }
  }
  
  return { left: x + 'px', top: y + 'px' };
});
function closeOverlays() {
  showMenu.value = false;
  showMiniCalendar.value = false;
  contextMenu.value.show = false;
}
function toggleMenu() {
  showMenu.value = !showMenu.value;
  if (showMenu.value) showMiniCalendar.value = false;
}
function toggleMiniCalendar() {
  showMiniCalendar.value = !showMiniCalendar.value;
  if (showMiniCalendar.value) showMenu.value = false;
}
function openSettings() {
  showSettings.value = true;
  showMenu.value = false;
}
function closeSettings() {
  showSettings.value = false;
  // 强制刷新日历网格以应用新设置（如每周开始于周一/周日）
  calendarKey.value++;
}
async function handleSaveSettings(settings: AppSettings) {
  await saveSettingsForMode(settings.theme_mode, settings);
  if (currentMode.value !== settings.theme_mode) {
    await switchMode(settings.theme_mode);
  }
  // 强制刷新日历网格和日程数据以应用新设置（如每周开始于周一/周日、显示模式等）
  calendarKey.value++;
  await refreshSchedules();
}
async function handleSwitchMode(mode: ThemeMode) {
  await switchMode(mode);
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
function handleUpdate(date: string, lines: { text: string; done: boolean }[]) {
  if (isLocked.value) return;

  // 保存撤销状态 - 获取当前日期的日程
  const currentSchedules = schedules.value.get(date) || [];
  const previousLines = currentSchedules
    .filter(s => s.id !== -1 && s.content.trim() !== '')
    .map(s => ({ text: s.content.trim(), done: !!s.is_done }));

  pushAction({
    type: 'updateLines',
    data: {
      date,
      previousLines,
    },
    timestamp: Date.now(),
  });

  updateScheduleLines(date, lines);
}
function handleSelectDate(day: number) {
  selectDate(day);
  showMiniCalendar.value = false;
}
function handleSwitchViewMode(mode: ViewMode) {
  switchViewMode(mode);
}
function handleNavigate(dateStr: string) {
  const targetDate = dayjs(dateStr);
  currentDate.value = targetDate;
  refreshSchedules();
}
function handleContextMenu(event: MouseEvent, date: string) {
  if (isLocked.value) return;
  contextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    date,
  };
}
function selectColor(color: string) {
  setCellColor(contextMenu.value.date, color);
  contextMenu.value.show = false;
}

async function handleToggleDone(schedule: any) {
  await toggleDoneWithUndo(schedule);
}

function handleEditDescription(schedule: any) {
  if (isLocked.value) return;
  editingSchedule.value = schedule;
  showDescriptionDialog.value = true;
}

async function handleDescriptionSave(scheduleId: number, description: string) {
  await updateScheduleDescription(scheduleId, description);
  showDescriptionDialog.value = false;
  editingSchedule.value = null;
}

function handleDescriptionCancel() {
  showDescriptionDialog.value = false;
  editingSchedule.value = null;
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

onMounted(async () => {
  await initDatabase();
  await refreshSchedules();
  await initSettings();
});

onUnmounted(() => {
});
</script>
<template>
  <div
    class="w-full h-full flex flex-col glass rounded-lg overflow-hidden relative"
    :style="{
      borderWidth: 'var(--cell-border-width)',
      borderStyle: 'solid',
      borderColor: 'var(--cell-border-color)'
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
    
    <SettingsPanel
      :visible="showSettings"
      :current-settings="currentSettings"
      :current-mode="currentMode"
      @close="closeSettings"
      @save="handleSaveSettings"
      @switch-mode="handleSwitchMode"
    />
    
    <div
      v-if="contextMenu.show"
      class="fixed z-50 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1"
      :style="contextMenuStyle"
    >
      <button
        v-for="color in colorOptions"
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
      @save="handleDescriptionSave"
      @cancel="handleDescriptionCancel"
    />

    <ToastContainer />
  </div>
</template>
