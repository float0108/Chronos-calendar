<script setup lang="ts">
import { ref, computed } from 'vue';
import dayjs from 'dayjs';
import CalendarCell from './CalendarCell.vue';
import { getCalendarDays } from '../../utils/date';
import { useSettings } from '../../composables/useSettings';
import type { Schedule } from '../../types';

const props = defineProps<{
  currentDate: dayjs.Dayjs;
  schedules: Map<string, Schedule[]>;
  isLocked: boolean;
  viewMode?: 'todo' | 'done';
}>();

const emit = defineEmits<{
  (e: 'reset', date: string, content: string | null): void;
  (e: 'update', date: string, lines: { text: string; done: boolean }[], viewMode?: 'todo' | 'done'): void;
  (e: 'navigate', direction: string): void;
  (e: 'contextmenu', event: MouseEvent, date: string): void;
  (e: 'toggleDone', schedule: Schedule): void;
  (e: 'editDescription', schedule: Schedule): void;
  (e: 'scheduleDrop', targetDate: string, scheduleId: number, sourceDate: string, viewMode?: 'todo' | 'done'): void;
}>();

const { getSetting } = useSettings();

const weekStartsOn = computed(() => (getSetting('week_starts_on') ?? 1) as 0 | 1);
const displayMode = computed(() => (getSetting('display_mode') ?? 'month') as 'month' | 'floating_weeks');
const floatingWeeksCount = computed(() => (getSetting('floating_weeks_count') ?? 3) as number);
const hideWeekends = computed(() => getSetting('hide_weekends') ?? false);

// 计算周末在周几列表中的索引
const weekendIndices = computed(() => {
  const start = weekStartsOn.value;
  // 周日是 0，周六是 6
  // 如果 weekStartsOn=1 (周一开头)，周日索引是 6，周六索引是 5
  // 如果 weekStartsOn=0 (周日开头)，周日索引是 0，周六索引是 6
  const sundayIndex = (0 - start + 7) % 7;
  const saturdayIndex = (6 - start + 7) % 7;
  return { sundayIndex, saturdayIndex };
});

const allWeekdays = computed(() => {
  const weekDays = ['日', '一', '二', '三', '四', '五', '六'];
  const start = weekStartsOn.value;
  const days = [];
  for (let i = 0; i < 7; i++) {
    days.push(weekDays[(start + i) % 7]);
  }
  return days;
});

// 根据是否隐藏周末过滤星期标题
const weekdays = computed(() => {
  if (!hideWeekends.value) return allWeekdays.value;
  return allWeekdays.value.filter((_, index) => {
    return index !== weekendIndices.value.sundayIndex && index !== weekendIndices.value.saturdayIndex;
  });
});

const allDays = computed(() => getCalendarDays(props.currentDate, weekStartsOn.value, displayMode.value, floatingWeeksCount.value));

// 根据是否隐藏周末过滤日期
const days = computed(() => {
  if (!hideWeekends.value) return allDays.value;
  return allDays.value.filter((_, index) => {
    const dayOfWeek = index % 7;
    return dayOfWeek !== weekendIndices.value.sundayIndex && dayOfWeek !== weekendIndices.value.saturdayIndex;
  });
});

// 计算列数
const gridCols = computed(() => hideWeekends.value ? 5 : 7);

const editingIndex = ref<number | null>(null);

function getDateSchedules(date: dayjs.Dayjs): Schedule[] {
  const dateStr = date.format('YYYY-MM-DD');
  return props.schedules.get(dateStr) || [];
}

function isToday(date: dayjs.Dayjs): boolean {
  return date.isSame(dayjs(), 'day');
}

function isCurrentMonth(date: dayjs.Dayjs): boolean {
  // 浮动周模式下，所有显示的单元格都是激活状态
  if (displayMode.value === 'floating_weeks') {
    return true;
  }
  // 整月模式下，只有当前月份的单元格是激活状态
  return date.isSame(props.currentDate, 'month');
}

function handleReset(date: string, content: string | null) {
  emit('reset', date, content);
}

function handleUpdate(date: string, lines: { text: string; done: boolean }[], viewMode?: 'todo' | 'done') {
  emit('update', date, lines, viewMode);
}

function handleCellContextMenu(event: MouseEvent, date: string) {
  emit('contextmenu', event, date);
}

function handleToggleDone(schedule: Schedule) {
  emit('toggleDone', schedule);
}

function handleEditDescription(schedule: Schedule) {
  emit('editDescription', schedule);
}

function handleCellFocus(index: number) {
  editingIndex.value = index;
}

function handleNavigate(direction: string) {
  if (editingIndex.value === null) return;

  let newIndex = editingIndex.value;

  if (direction === 'ArrowRight') {
    newIndex = Math.min(days.value.length - 1, newIndex + 1);
  } else if (direction === 'ArrowLeft') {
    newIndex = Math.max(0, newIndex - 1);
  } else if (direction === 'ArrowDown') {
    newIndex = Math.min(days.value.length - 1, newIndex + gridCols.value);
  } else if (direction === 'ArrowUp') {
    newIndex = Math.max(0, newIndex - gridCols.value);
  }

  if (newIndex !== editingIndex.value) {
    const date = days.value[newIndex];
    if (date) {
      emit('navigate', date.format('YYYY-MM-DD'));
    }
  }
}

function handleScheduleDrop(targetDate: string, event: DragEvent) {
  if (!event.dataTransfer) return;

  const data = event.dataTransfer.getData('application/json');
  if (!data) return;

  try {
    const scheduleData = JSON.parse(data);
    emit('scheduleDrop', targetDate, scheduleData.id, scheduleData.sourceDate, scheduleData.viewMode);
  } catch (e) {
    console.error('Failed to parse drag data:', e);
  }
}
</script>

<template>
  <div class="calendar-grid flex-1 flex flex-col p-2 min-h-0">
    <div class="grid mb-1 shrink-0" :style="{ gridTemplateColumns: `repeat(${gridCols}, minmax(0, 1fr))`, gap: 'var(--cell-gap)' }">
      <div
        v-for="day in weekdays"
        :key="day"
        class="text-center py-1 font-medium text-gray-900"
        style="font-size: var(--font-size-xs);"
      >
        {{ day }}
      </div>
    </div>

    <div class="grid flex-1 min-h-0" :style="{ gridTemplateColumns: `repeat(${gridCols}, minmax(0, 1fr))`, gridTemplateRows: `repeat(${Math.ceil(days.length / gridCols)}, minmax(0, 1fr))`, gap: 'var(--cell-gap)' }">
      <CalendarCell
        v-for="(date, index) in days"
        :key="date.format('YYYY-MM-DD')"
        :date="date"
        :schedules="getDateSchedules(date)"
        :is-today="isToday(date)"
        :is-current-month="isCurrentMonth(date)"
        :is-locked="isLocked"
        :view-mode="viewMode"
        class="h-full overflow-hidden"
        @reset="handleReset"
        @update="handleUpdate"
        @focus="handleCellFocus(index)"
        @navigate="handleNavigate"
        @contextmenu="handleCellContextMenu($event, date.format('YYYY-MM-DD'))"
        @toggle-done="handleToggleDone"
        @edit-description="handleEditDescription"
        @schedule-drop="handleScheduleDrop"
      />
    </div>
  </div>
</template>
