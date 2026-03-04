import { ref, computed } from 'vue';
import dayjs from 'dayjs';
import type { Schedule, ViewMode } from '../types';
import { useDatabase } from './useDatabase';
import { useToast } from './useToast';
import { schedulesToCSV, exportToFile } from '../utils/export';

export function useSchedules() {
  const { loadSchedules, loadTodoSchedules, loadDoneSchedules, saveSchedule, deleteSchedulesByDate, updateScheduleColor, loadAllSchedules, importSchedules, importCellColors, clearAllData, toggleScheduleStatus } = useDatabase();

  // 导出 saveSchedule 供撤销功能使用
  const _saveSchedule = saveSchedule;
  const { showError, showSuccess } = useToast();

  const schedules = ref<Map<string, Schedule[]>>(new Map());
  const currentDate = ref(dayjs());
  const viewMode = ref<ViewMode>('todo');

  const monthStr = computed(() => currentDate.value.format('YYYY-MM'));

  async function refreshSchedules(): Promise<void> {
    let result: Schedule[];

    // 根据视图模式选择加载函数
    if (viewMode.value === 'todo') {
      result = await loadTodoSchedules(monthStr.value);
    } else if (viewMode.value === 'done') {
      result = await loadDoneSchedules(monthStr.value);
    } else {
      result = await loadSchedules(monthStr.value);
    }

    schedules.value.clear();
    result.forEach(schedule => {
      const dateStr = schedule.create_date;
      if (!schedules.value.has(dateStr)) {
        schedules.value.set(dateStr, []);
      }
      schedules.value.get(dateStr)!.push(schedule);
    });
  }

  function switchViewMode(mode: ViewMode): void {
    viewMode.value = mode;
    refreshSchedules();
  }

  function getDateSchedules(date: dayjs.Dayjs): Schedule[] {
    const dateStr = date.format('YYYY-MM-DD');
    return schedules.value.get(dateStr) || [];
  }

  async function addSchedule(date: string, content: string, replace: boolean = false): Promise<void> {
    if (!content.trim()) return;
    
    try {
      if (replace) {
        await deleteSchedulesByDate(date);
      }
      
      await saveSchedule(date, content);
      await refreshSchedules();
    } catch (error) {
      showError('保存日程失败，请重试');
    }
  }

  async function resetSchedule(date: string, content: string | null): Promise<void> {
    try {
      if (content === null) {
        await deleteSchedulesByDate(date);
      } else {
        if (content.trim()) {
          await deleteSchedulesByDate(date);
          await saveSchedule(date, content);
        }
      }
      await refreshSchedules();
    } catch (error) {
      showError('保存日程失败，请重试');
    }
  }

  async function updateScheduleLines(date: string, lines: { text: string; done: boolean }[]): Promise<void> {
    const existingSchedules = getDateSchedules(dayjs(date));
    // 过滤掉虚拟记录（只有颜色没有内容的记录 id 为 -1）
    const validExistingSchedules = existingSchedules.filter(s => s.id !== -1);

    const existingContent = validExistingSchedules.map(s => ({
      text: s.content,
      done: !!s.is_done
    }));

    // 比较内容是否变化（过滤掉空行后）
    const validLines = lines.filter(l => l.text.trim() !== '');
    if (JSON.stringify(existingContent) === JSON.stringify(validLines)) {
      return;
    }

    try {
      // 删除该日期的所有日程内容（颜色保留在 cell_metadata 表中）
      await deleteSchedulesByDate(date);

      // 获取当前日期作为 done_date
      const now = new Date();
      const today = now.getFullYear() + '-' +
        String(now.getMonth() + 1).padStart(2, '0') + '-' +
        String(now.getDate()).padStart(2, '0');

      // 插入新的日程行
      for (const line of validLines) {
        await saveSchedule(date, line.text.trim(), line.done, line.done ? today : undefined);
      }
      await refreshSchedules();
    } catch (error) {
      showError('保存日程失败，请重试');
    }
  }

  function prevMonth(): void {
    currentDate.value = currentDate.value.subtract(1, 'month');
    refreshSchedules();
  }

  function nextMonth(): void {
    currentDate.value = currentDate.value.add(1, 'month');
    refreshSchedules();
  }

  function goToToday(): void {
    currentDate.value = dayjs();
    refreshSchedules();
  }

  function selectDate(day: number): void {
    currentDate.value = currentDate.value.date(day);
    refreshSchedules();
  }

  async function setCellColor(date: string, color: string): Promise<void> {
    try {
      await updateScheduleColor(date, color);
      await refreshSchedules();
      showSuccess('设置颜色成功');
    } catch (error) {
      showError('设置颜色失败，请重试');
    }
  }

  async function exportAllSchedules(): Promise<void> {
    try {
      const { schedules, cellColors } = await loadAllSchedules();

      if (schedules.length === 0 && cellColors.size === 0) {
        showError('没有日程数据可导出');
        return;
      }

      const csvData = schedulesToCSV(schedules, cellColors);
      const filename = `chronos-schedules-${dayjs().format('YYYY-MM-DD')}.csv`;
      const success = await exportToFile(csvData, filename);

      if (success) {
        showSuccess('导出成功');
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : '导出失败，请重试';
      showError(message);
    }
  }

  async function importSchedulesFromData(schedules: Schedule[], cellColors: { date: string, color: string }[], mode: 'merge' | 'overwrite'): Promise<{ success: boolean, scheduleStats?: { inserted: number, updated: number }, colorStats?: { inserted: number, updated: number } }> {
    try {
      console.log('Importing data:', { mode, schedulesCount: schedules.length, cellColorsCount: cellColors.length });
      console.log('Sample schedule:', schedules[0]);
      console.log('Sample cellColor:', cellColors[0]);

      if (mode === 'overwrite') {
        await clearAllData();
      }

      let scheduleStats = { inserted: 0, updated: 0 };
      let colorStats = { inserted: 0, updated: 0 };

      if (schedules.length > 0) {
        scheduleStats = await importSchedules(schedules, mode === 'merge');
        console.log('Schedule import result:', scheduleStats);
      }

      if (cellColors.length > 0) {
        colorStats = await importCellColors(cellColors);
        console.log('Color import result:', colorStats);
      }

      await refreshSchedules();
      return { success: true, scheduleStats, colorStats };
    } catch (error) {
      console.error('Import error:', error);
      const message = error instanceof Error ? error.message : '导入失败，请重试';
      showError(message);
      return { success: false };
    }
  }

  return {
    schedules,
    currentDate,
    viewMode,
    getDateSchedules,
    refreshSchedules,
    switchViewMode,
    addSchedule,
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
    saveSchedule: _saveSchedule,
  };
}
