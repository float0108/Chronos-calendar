import dayjs from 'dayjs';

export function getCalendarDays(currentDate: dayjs.Dayjs, weekStartsOn: 0 | 1 = 1): dayjs.Dayjs[] {
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
}

export function getMiniCalendarDays(year: number, month: number): number[] {
  const daysInMonth = dayjs(new Date(year, month, 1)).daysInMonth();
  return Array.from({ length: daysInMonth }, (_, i) => i + 1);
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
