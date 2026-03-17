import dayjs from 'dayjs';

export function getCalendarDays(
  currentDate: dayjs.Dayjs,
  weekStartsOn: 0 | 1 = 1,
  displayMode: 'month' | 'floating_weeks' = 'month',
  floatingWeeksCount: number = 3
): dayjs.Dayjs[] {
  if (displayMode === 'month') {
    // 整月显示模式
    const year = currentDate.year();
    const month = currentDate.month();

    const firstDayOfMonth = dayjs(new Date(year, month, 1));
    const firstDayWeekday = firstDayOfMonth.day();

    const adjustedFirstDay = (firstDayWeekday - weekStartsOn + 7) % 7;

    const days: dayjs.Dayjs[] = [];

    for (let i = adjustedFirstDay; i > 0; i--) {
      days.push(firstDayOfMonth.subtract(i, 'day'));
    }

    const daysInMonth = firstDayOfMonth.daysInMonth();
    for (let i = 0; i < daysInMonth; i++) {
      days.push(firstDayOfMonth.add(i, 'day'));
    }

    const remaining = 42 - days.length;
    const lastDayOfMonth = firstDayOfMonth.add(daysInMonth - 1, 'day');
    for (let i = 1; i <= remaining; i++) {
      days.push(lastDayOfMonth.add(i, 'day'));
    }

    return days;
  } else {
    // 浮动周显示模式
    const currentDayOfWeek = currentDate.day();
    const adjustedCurrentDay = (currentDayOfWeek - weekStartsOn + 7) % 7;
    const startOfWeek = currentDate.subtract(adjustedCurrentDay, 'day');

    let weeksBefore: number;

    if (floatingWeeksCount % 2 === 1) {
      // 奇数：前 (n-1)/2 周，后 (n-1)/2 周
      const halfWeeks = (floatingWeeksCount - 1) / 2;
      weeksBefore = halfWeeks;
    } else {
      // 偶数：前 n/2-1 周，后 n/2+1 周
      weeksBefore = floatingWeeksCount / 2 - 1;
    }

    const startDate = startOfWeek.subtract(weeksBefore, 'week');
    const totalDays = floatingWeeksCount * 7;

    const days: dayjs.Dayjs[] = [];
    for (let i = 0; i < totalDays; i++) {
      days.push(startDate.add(i, 'day'));
    }

    return days;
  }
}

export function getMiniCalendarDays(
  year: number,
  month: number,
  weekStartsOn: 0 | 1 = 1
): (number | null)[] {
  const firstDayOfMonth = dayjs(new Date(year, month, 1));
  const daysInMonth = firstDayOfMonth.daysInMonth();
  const firstDayWeekday = firstDayOfMonth.day();

  // 计算第一天前面需要填充的空白格子数量
  const leadingBlanks = (firstDayWeekday - weekStartsOn + 7) % 7;

  const days: (number | null)[] = [];

  // 添加前置空白
  for (let i = 0; i < leadingBlanks; i++) {
    days.push(null);
  }

  // 添加实际日期
  for (let i = 1; i <= daysInMonth; i++) {
    days.push(i);
  }

  return days;
}

export function formatMonthYear(date: dayjs.Dayjs): string {
  return date.format('YYYY年 M月');
}

export function isSameDay(a: dayjs.Dayjs, b: dayjs.Dayjs): boolean {
  return a.isSame(b, 'day');
}

export function isSameMonth(a: dayjs.Dayjs, b: dayjs.Dayjs): boolean {
  return a.isSame(b, 'month');
}
