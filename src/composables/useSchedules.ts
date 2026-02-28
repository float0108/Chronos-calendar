import { ref, computed } from 'vue';
import dayjs from 'dayjs';
import type { Schedule } from '../types';
import { useDatabase } from './useDatabase';
import { useToast } from './useToast';

export function useSchedules() {
  const { loadSchedules, saveSchedule, deleteSchedulesByDate, updateScheduleColor } = useDatabase();
  const { showError, showSuccess } = useToast();
  
  const schedules = ref<Map<string, Schedule[]>>(new Map());
  const currentDate = ref(dayjs());

  const monthStr = computed(() => currentDate.value.format('YYYY-MM'));

  async function refreshSchedules(): Promise<void> {
    const result = await loadSchedules(monthStr.value);
    
    schedules.value.clear();
    result.forEach(schedule => {
      const dateStr = schedule.date;
      if (!schedules.value.has(dateStr)) {
        schedules.value.set(dateStr, []);
      }
      schedules.value.get(dateStr)!.push(schedule);
    });
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
      
      // 插入新的日程行
      for (const line of validLines) {
        await saveSchedule(date, line.text.trim(), line.done);
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

  return {
    schedules,
    currentDate,
    getDateSchedules,
    refreshSchedules,
    addSchedule,
    resetSchedule,
    updateScheduleLines,
    setCellColor,
    prevMonth,
    nextMonth,
    goToToday,
    selectDate,
  };
}
