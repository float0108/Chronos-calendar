<script setup lang="ts">
import { ref, computed } from 'vue';
import dayjs from 'dayjs';
import CalendarCell from './CalendarCell.vue';
import { getCalendarDays } from '../utils/date';
import { useSettings } from '../composables/useSettings';
import type { Schedule } from '../types';

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
}>();

const { getSetting } = useSettings();

const weekStartsOn = computed(() => (getSetting('week_starts_on') ?? 1) as 0 | 1);
const displayMode = computed(() => (getSetting('display_mode') ?? 'month') as 'month' | 'floating_weeks');
const floatingWeeksCount = computed(() => (getSetting('floating_weeks_count') ?? 3) as number);

const weekdays = computed(() => {
  const weekDays = ['日', '一', '二', '三', '四', '五', '六'];
  const start = weekStartsOn.value;
  const days = [];
  for (let i = 0; i < 7; i++) {
    days.push(weekDays[(start + i) % 7]);
  }
  return days;
});

const days = computed(() => getCalendarDays(props.currentDate, weekStartsOn.value, displayMode.value, floatingWeeksCount.value));

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
    newIndex = Math.min(41, newIndex + 1);
  } else if (direction === 'ArrowLeft') {
    newIndex = Math.max(0, newIndex - 1);
  } else if (direction === 'ArrowDown') {
    newIndex = Math.min(41, newIndex + 7);
  } else if (direction === 'ArrowUp') {
    newIndex = Math.max(0, newIndex - 7);
  }
  
  if (newIndex !== editingIndex.value) {
    const date = days.value[newIndex];
    if (date) {
      emit('navigate', date.format('YYYY-MM-DD'));
    }
  }
}
</script>

<template>
  <div class="calendar-grid flex-1 flex flex-col p-2 min-h-0">
    <div class="grid grid-cols-7 mb-1 shrink-0" style="gap: var(--cell-gap)">
      <div 
        v-for="day in weekdays" 
        :key="day"
        class="text-center py-1 font-medium text-gray-900"
        style="font-size: var(--font-size-xs);"
      >
        {{ day }}
      </div>
    </div>
    
    <div class="grid grid-cols-7 flex-1 min-h-0" :style="{ gridTemplateRows: `repeat(${Math.ceil(days.length / 7)}, minmax(0, 1fr))`, gap: 'var(--cell-gap)' }">
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
      />
    </div>
  </div>
</template>