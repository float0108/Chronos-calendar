<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import dayjs from 'dayjs';
import CalendarHeader from './components/CalendarHeader.vue';
import CalendarGrid from './components/CalendarGrid.vue';
import ResizeHandles from './components/ResizeHandles.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import ToastContainer from './components/ToastContainer.vue';
import { useDatabase } from './composables/useDatabase';
import { useSchedules } from './composables/useSchedules';
import { useSettings } from './composables/useSettings';
import { closeWindow, setWindowLocked } from './utils/window';
import type { AppSettings, ThemeMode } from './types';
const { initDatabase } = useDatabase();
const {
  schedules,
  currentDate,
  refreshSchedules,
  resetSchedule,
  updateScheduleLines,
  setCellColor,
  prevMonth,
  nextMonth,
  goToToday,
  selectDate,
} = useSchedules();
const {
  currentSettings,
  currentMode,
  initSettings,
  saveSettingsForMode,
  switchMode,
} = useSettings();
const calendarKey = ref(0);
const showMenu = ref(false);
const showMiniCalendar = ref(false);
const showSettings = ref(false);
const isLocked = ref(false);
const contextMenu = ref<{ show: boolean; x: number; y: number; date: string }>({
  show: false,
  x: 0,
  y: 0,
  date: '',
});
const colorOptions = [
  { name: '默认', value: '' },
  { name: '红', value: '#fecaca' },
  { name: '橙', value: '#fed7aa' },
  { name: '黄', value: '#fef08a' },
  { name: '绿', value: '#bbf7d0' },
  { name: '蓝', value: '#bfdbfe' },
  { name: '紫', value: '#ddd6fe' },
  { name: '粉', value: '#fbcfe8' },
];
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
  if (isLocked.value) return;
  showMenu.value = !showMenu.value;
  if (showMenu.value) showMiniCalendar.value = false;
}
function toggleMiniCalendar() {
  if (isLocked.value) return;
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
  // 强制刷新日历网格以应用新设置（如每周开始于周一/周日）
  calendarKey.value++;
}
async function handleSwitchMode(mode: ThemeMode) {
  await switchMode(mode);
}
async function quitApp() {
  await closeWindow();
}
function handleReset(date: string, content: string | null) {
  if (isLocked.value) return;
  resetSchedule(date, content);
}
function handleUpdate(date: string, lines: { text: string; done: boolean }[]) {
  if (isLocked.value) return;
  updateScheduleLines(date, lines);
}
function handleSelectDate(day: number) {
  if (isLocked.value) return;
  selectDate(day);
  showMiniCalendar.value = false;
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
function toggleLock() {
  isLocked.value = !isLocked.value;
  setWindowLocked(isLocked.value);
}
onMounted(async () => {
  await initDatabase();
  await refreshSchedules();
  await initSettings();
});
</script>
<template>
  <div class="w-full h-full flex flex-col glass rounded-lg overflow-hidden relative">
    <ResizeHandles :is-locked="isLocked" />
    
    <CalendarHeader
      :current-date="currentDate"
      :show-mini-calendar="showMiniCalendar"
      :show-menu="showMenu"
      :is-locked="isLocked"
      @prev-month="!isLocked && prevMonth()"
      @next-month="!isLocked && nextMonth()"
      @go-today="!isLocked && goToToday()"
      @toggle-mini-calendar="toggleMiniCalendar"
      @toggle-menu="toggleMenu"
      @select-date="handleSelectDate"
      @settings="openSettings"
      @quit="quitApp"
      @toggle-lock="toggleLock"
    />
    
    <CalendarGrid
      :key="calendarKey"
      :current-date="currentDate"
      :schedules="schedules"
      :is-locked="isLocked"
      @reset="handleReset"
      @update="handleUpdate"
      @navigate="handleNavigate"
      @contextmenu="handleContextMenu"
    />
    
    <div 
      v-if="showMenu || showMiniCalendar" 
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
    
    <ToastContainer />
  </div>
</template>
