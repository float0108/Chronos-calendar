<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import dayjs from 'dayjs';
import CalendarHeader from './components/CalendarHeader.vue';
import CalendarGrid from './components/CalendarGrid.vue';
import ResizeHandles from './components/ResizeHandles.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import ToastContainer from './components/ToastContainer.vue';
import ImportDialog from './components/ImportDialog.vue';
import { useDatabase } from './composables/useDatabase';
import { useSchedules } from './composables/useSchedules';
import { useSettings } from './composables/useSettings';
import { useToast } from './composables/useToast';
import { useUndoHistory } from './composables/useUndoHistory';
import { closeWindow, setWindowLocked } from './utils/window';
import { importFromFile, csvToSchedules } from './utils/export';
import type { AppSettings, ThemeMode, ViewMode } from './types';
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
} = useSchedules();
const {
  currentSettings,
  currentMode,
  initSettings,
  saveSettingsForMode,
  switchMode,
} = useSettings();
const { showSuccess, showError } = useToast();
const { pushAction, popAction, pushRedo, popRedo, canUndo, canRedo } = useUndoHistory();
const calendarKey = ref(0);
const showMenu = ref(false);
const showMiniCalendar = ref(false);
const showSettings = ref(false);
const showImportDialog = ref(false);
const isLocked = ref(false);
const pendingImportRecordCount = ref(0);
const pendingImportData = ref<{ schedules: any[], cellColors: { date: string, color: string }[] } | null>(null);
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
async function handleExport() {
  showMenu.value = false;
  await exportAllSchedules();
}
async function handleImport() {
  showMenu.value = false;

  try {
    const csvContent = await importFromFile();

    if (!csvContent) {
      // User cancelled
      return;
    }

    console.log('CSV content length:', csvContent.length);
    console.log('CSV first 500 chars:', csvContent.substring(0, 500));

    const { schedules, cellColors } = csvToSchedules(csvContent);
    console.log('Parsed data:', { schedules, cellColors });

    pendingImportRecordCount.value = schedules.length + cellColors.length;

    if (pendingImportRecordCount.value === 0) {
      showError('CSV 文件中没有数据');
      return;
    }

    // Store the parsed data for later use
    pendingImportData.value = { schedules, cellColors };

    // Show import dialog to ask user for strategy
    showImportDialog.value = true;
  } catch (error) {
    console.error('Import error:', error);
    const message = error instanceof Error ? error.message : '导入失败，请重试';
    showError(message);
  }
}
async function handleImportMerge() {
  showImportDialog.value = false;

  if (!pendingImportData.value) {
    showError('导入数据丢失，请重试');
    return;
  }

  const { schedules, cellColors } = pendingImportData.value;
  pendingImportData.value = null;

  const result = await importSchedulesFromData(schedules, cellColors, 'merge');
  if (result.success && result.scheduleStats && result.colorStats) {
    const messages = [];
    if (result.scheduleStats.inserted > 0) {
      messages.push(`新增 ${result.scheduleStats.inserted} 条日程`);
    }
    if (result.scheduleStats.updated > 0) {
      messages.push(`更新 ${result.scheduleStats.updated} 条日程`);
    }
    if (result.colorStats.inserted > 0) {
      messages.push(`新增 ${result.colorStats.inserted} 个颜色标记`);
    }
    if (result.colorStats.updated > 0) {
      messages.push(`更新 ${result.colorStats.updated} 个颜色标记`);
    }

    if (messages.length > 0) {
      showSuccess(`导入成功：${messages.join('，')}`);
    } else {
      showSuccess('导入成功：无数据变化');
    }
  }
}
async function handleImportOverwrite() {
  showImportDialog.value = false;

  if (!pendingImportData.value) {
    showError('导入数据丢失，请重试');
    return;
  }

  const { schedules, cellColors } = pendingImportData.value;
  pendingImportData.value = null;

  const result = await importSchedulesFromData(schedules, cellColors, 'overwrite');
  if (result.success && result.scheduleStats && result.colorStats) {
    const messages = [];
    if (result.scheduleStats.inserted > 0) {
      messages.push(`${result.scheduleStats.inserted} 条日程`);
    }
    if (result.colorStats.inserted > 0) {
      messages.push(`${result.colorStats.inserted} 个颜色标记`);
    }

    if (messages.length > 0) {
      showSuccess(`导入成功：${messages.join('，')}`);
    } else {
      showSuccess('导入成功');
    }
  }
}
function handleImportCancel() {
  showImportDialog.value = false;
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
  if (isLocked.value) return;
  selectDate(day);
  showMiniCalendar.value = false;
}
function handleSwitchViewMode(mode: ViewMode) {
  if (isLocked.value) return;
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
  // 保存撤销状态
  pushAction({
    type: 'toggleDone',
    data: {
      id: schedule.id,
      previousState: schedule.is_done,
    },
    timestamp: Date.now(),
  });

  // 切换完成状态（传入切换后的状态，而不是当前状态）
  await toggleScheduleStatus(schedule.id, !schedule.is_done);
  await refreshSchedules();
}

async function handleUndo() {
  const action = popAction();
  if (!action) return;

  try {
    switch (action.type) {
      case 'toggleDone': {
        const { id, previousState } = action.data;
        await toggleScheduleStatus(id, !previousState);
        await refreshSchedules();
        showSuccess('已撤销：切换完成状态');
        break;
      }
      case 'updateLines': {
        const { date, previousLines } = action.data;
        // 获取当前状态用于重做
        const currentSchedules = schedules.value.get(date) || [];
        const currentLines = currentSchedules
          .filter(s => s.id !== -1 && s.content.trim() !== '')
          .map(s => ({ text: s.content.trim(), done: !!s.is_done }));
        // 保存当前状态到重做历史
        pushRedo({
          type: 'updateLines',
          data: { date, previousLines: currentLines },
          timestamp: Date.now(),
        });
        await updateScheduleLines(date, previousLines);
        await refreshSchedules();
        showSuccess('已撤销：编辑操作');
        break;
      }
      case 'deleteSchedule': {
        const { date, previousSchedules } = action.data;
        // 恢复删除的日程
        for (const schedule of previousSchedules) {
          await saveSchedule(date, schedule.content, schedule.is_done, schedule.done_date, schedule.description);
        }
        await refreshSchedules();
        showSuccess('已撤销：删除操作');
        break;
      }
    }
  } catch (error) {
    console.error('Undo failed:', error);
    showError('撤销失败');
  }
}

async function handleRedo() {
  const action = popRedo();
  if (!action) return;

  try {
    switch (action.type) {
      case 'updateLines': {
        const { date, previousLines } = action.data;
        await updateScheduleLines(date, previousLines);
        await refreshSchedules();
        showSuccess('已重做：编辑操作');
        break;
      }
      default:
        console.warn('Unsupported redo action type:', action.type);
    }
  } catch (error) {
    console.error('Redo failed:', error);
    showError('重做失败');
  }
}

function toggleLock() {
  isLocked.value = !isLocked.value;
  setWindowLocked(isLocked.value);
}

function handleKeydown(event: KeyboardEvent) {
  // 全局撤销/重做仅在非输入状态下由按钮触发
  // 这里不处理键盘快捷键
}

onMounted(async () => {
  await initDatabase();
  await refreshSchedules();
  await initSettings();

  // 监听键盘事件
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});
</script>
<template>
  <div class="w-full h-full flex flex-col glass rounded-lg overflow-hidden relative" @contextmenu.prevent>
    <ResizeHandles :is-locked="isLocked" />
    
    <CalendarHeader
      :current-date="currentDate"
      :show-mini-calendar="showMiniCalendar"
      :show-menu="showMenu"
      :is-locked="isLocked"
      :can-undo="canUndo()"
      :can-redo="canRedo()"
      :view-mode="viewMode"
      @prev-month="!isLocked && prevMonth()"
      @next-month="!isLocked && nextMonth()"
      @go-today="!isLocked && goToToday()"
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
      @reset="handleReset"
      @update="handleUpdate"
      @navigate="handleNavigate"
      @contextmenu="handleContextMenu"
      @toggle-done="handleToggleDone"
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

    <ImportDialog
      :visible="showImportDialog"
      :record-count="pendingImportRecordCount"
      @merge="handleImportMerge"
      @overwrite="handleImportOverwrite"
      @cancel="handleImportCancel"
    />

    <ToastContainer />
  </div>
</template>
