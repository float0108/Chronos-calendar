import { ref, computed } from 'vue';
import dayjs from 'dayjs';
import type { Schedule, ViewMode, BatchTaskConfig } from '../types';
import { useDatabase } from './useDatabase';
import { useToast } from './useToast';
import { useSettings } from './useSettings';
import { schedulesToCSV, exportToFile } from '../utils/export';
import { getCalendarDays } from '../utils/date';

export function useSchedules() {
  const { loadSchedules, loadTodoSchedules, loadDoneSchedules, saveSchedule, deleteSchedule, deleteSchedulesByDate, updateScheduleColor, loadAllSchedules, importSchedules, importCellColors, clearAllData, toggleScheduleStatus, updateScheduleDescription: dbUpdateDescription, updateScheduleContent, updateScheduleDate: dbUpdateDate, updateScheduleFatherTask: dbUpdateFatherTask } = useDatabase();

  // 导出 saveSchedule 供撤销功能使用
  const _saveSchedule = saveSchedule;
  const { showError, showSuccess } = useToast();
  const { getSetting } = useSettings();

  const schedules = ref<Map<string, Schedule[]>>(new Map());
  const currentDate = ref(dayjs());
  const viewMode = ref<ViewMode>('todo');

  const monthStr = computed(() => currentDate.value.format('YYYY-MM'));

  // 获取日历视图的日期范围
  const dateRange = computed(() => {
    const weekStartsOn = getSetting('week_starts_on') ?? 1;
    const displayMode = getSetting('display_mode') ?? 'month';
    const floatingWeeksCount = getSetting('floating_weeks_count') ?? 3;

    const days = getCalendarDays(currentDate.value, weekStartsOn as 0 | 1, displayMode, floatingWeeksCount);
    if (days.length === 0) return { startDate: monthStr.value, endDate: monthStr.value };
    const startDate = days[0].format('YYYY-MM-DD');
    const endDate = days[days.length - 1].format('YYYY-MM-DD');
    return { startDate, endDate };
  });

  async function refreshSchedules(): Promise<void> {
    let result: Schedule[];

    const { startDate, endDate } = dateRange.value;

    // 根据视图模式选择加载函数
    if (viewMode.value === 'todo') {
      result = await loadTodoSchedules(startDate, endDate);
    } else if (viewMode.value === 'done') {
      result = await loadDoneSchedules(startDate, endDate);
    } else {
      result = await loadSchedules(startDate, endDate);
    }

    schedules.value.clear();
    result.forEach(schedule => {
      // todo 视图按 create_date 组织，done 视图按 done_date 组织
      const dateStr = viewMode.value === 'done' ? schedule.done_date : schedule.create_date;
      if (!dateStr) return; // 跳过没有有效日期的记录
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

  // 暂时未使用，但保留以便将来使用
  void addSchedule;

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

  async function updateScheduleLines(date: string, lines: { id?: number; text: string; done: boolean }[], isDoneView: boolean = false): Promise<void> {
    const existingSchedules = getDateSchedules(dayjs(date));
    // 过滤掉虚拟记录（只有颜色没有内容的记录 id 为 -1）
    const validExistingSchedules = existingSchedules.filter(s => s.id !== -1);

    // 过滤掉空行
    const validLines = lines.filter(l => l.text.trim() !== '');

    // 检查是否有变化
    const existingMap = new Map(validExistingSchedules.map(s => [s.id, { text: s.content.trim(), done: !!s.is_done }]));

    let hasChanges = validLines.length !== validExistingSchedules.length;
    if (!hasChanges) {
      // 检查每一行的内容和完成状态是否匹配
      for (const line of validLines) {
        if (line.id !== undefined) {
          const existing = existingMap.get(line.id);
          if (!existing || existing.text !== line.text.trim() || existing.done !== line.done) {
            hasChanges = true;
            break;
          }
        } else {
          // 新行
          hasChanges = true;
          break;
        }
      }
    }

    if (!hasChanges) return;

    try {
      // 找出需要删除的日程 id
      const lineIds = new Set(validLines.filter(l => l.id !== undefined).map(l => l.id));
      const toDelete = validExistingSchedules.filter(s => !lineIds.has(s.id));

      // 删除不再存在的日程
      for (const schedule of toDelete) {
        await deleteSchedule(schedule.id!);
      }

      // 更新或创建日程
      for (const line of validLines) {
        if (line.id !== undefined) {
          // 更新现有日程的内容
          const existing = existingMap.get(line.id);
          if (existing && existing.text !== line.text.trim()) {
            await updateScheduleContent(line.id, line.text.trim());
          }

          // 如果完成状态改变，更新完成状态
          if (existing && existing.done !== line.done) {
            await toggleScheduleStatus(line.id, line.done);
          }
        } else {
          // 创建新日程
          // 在 done 视图下，create_date 和 done_date 都设为当前单元格日期
          if (isDoneView) {
            await saveSchedule(date, line.text.trim(), true, date);
          } else {
            const now = new Date();
            const today = now.getFullYear() + '-' +
              String(now.getMonth() + 1).padStart(2, '0') + '-' +
              String(now.getDate()).padStart(2, '0');
            await saveSchedule(date, line.text.trim(), line.done, line.done ? today : undefined);
          }
        }
      }

      await refreshSchedules();
    } catch (error) {
      showError('保存日程失败，请重试');
    }
  }

  function navigateMonth(direction: -1 | 1): void {
    const displayMode = getSetting('display_mode') ?? 'month';
    const floatingWeeksCount = getSetting('floating_weeks_count') ?? 3;

    if (displayMode === 'floating_weeks') {
      // 浮动周模式：翻 floatingWeeksCount 周
      const amount = floatingWeeksCount * direction;
      currentDate.value = direction > 0
        ? currentDate.value.add(amount, 'week')
        : currentDate.value.subtract(-amount, 'week');
    } else {
      // 整月模式：翻一个月
      currentDate.value = direction > 0
        ? currentDate.value.add(1, 'month')
        : currentDate.value.subtract(1, 'month');
    }
    refreshSchedules();
  }

  function prevMonth(): void {
    navigateMonth(-1);
  }

  function nextMonth(): void {
    navigateMonth(1);
  }

  function goToToday(): void {
    currentDate.value = dayjs();
    refreshSchedules();
  }

  function selectDate(date: dayjs.Dayjs): void {
    currentDate.value = date;
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

  async function updateScheduleDescription(scheduleId: number, description: string): Promise<void> {
    await dbUpdateDescription(scheduleId, description);
    await refreshSchedules();
  }

  async function updateScheduleDate(scheduleId: number, field: 'create_date' | 'done_date', date: string): Promise<void> {
    await dbUpdateDate(scheduleId, field, date);
    await refreshSchedules();
  }

  async function updateScheduleFatherTask(scheduleId: number, fatherTask: number | null): Promise<void> {
    await dbUpdateFatherTask(scheduleId, fatherTask);
    await refreshSchedules();
  }

  async function batchAddSchedules(config: BatchTaskConfig): Promise<{ success: boolean; count: number }> {
    try {
      const start = dayjs(config.startDate);
      const end = dayjs(config.endDate);
      const dates: string[] = [];

      // 生成日期列表
      let current = start;
      while (current.isBefore(end) || current.isSame(end, 'day')) {
        dates.push(current.format('YYYY-MM-DD'));
        switch (config.cycleType) {
          case 'day':
            current = current.add(config.cycleCount, 'day');
            break;
          case 'week':
            current = current.add(config.cycleCount, 'week');
            break;
          case 'month':
            current = current.add(config.cycleCount, 'month');
            break;
        }
      }

      // 批量插入日程
      for (const date of dates) {
        await saveSchedule(date, config.title, false, undefined, config.description);
      }

      await refreshSchedules();
      showSuccess(`成功创建 ${dates.length} 个任务`);
      return { success: true, count: dates.length };
    } catch (error) {
      const message = error instanceof Error ? error.message : '批量添加任务失败，请重试';
      showError(message);
      return { success: false, count: 0 };
    }
  }

  return {
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
    saveSchedule: _saveSchedule,
    deleteSchedule,
    updateScheduleDescription,
    updateScheduleContent,
    updateScheduleDate,
    updateScheduleFatherTask,
    batchAddSchedules,
  };
}
