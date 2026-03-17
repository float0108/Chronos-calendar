<script setup lang="ts">
import { computed } from 'vue';
import { ChevronLeft, ChevronRight } from 'lucide-vue-next';
import dayjs from 'dayjs';
import { getMiniCalendarDays } from '../../utils/date';
import { useSettings } from '../../composables/useSettings';

const props = defineProps<{
  currentDate: dayjs.Dayjs;
  visible: boolean;
  position?: { top: number; left: number };
}>();

const emit = defineEmits<{
  (e: 'select', day: number): void;
  (e: 'prevMonth'): void;
  (e: 'nextMonth'): void;
  (e: 'close'): void;
}>();

const { getSetting, currentMode } = useSettings();

const weekStartsOn = computed(() => (getSetting('week_starts_on') ?? 1) as 0 | 1);
const primaryColor = computed(() => getSetting('primary_color') ?? '#3b82f6');
const themeMode = computed(() => currentMode.value);

const days = computed(() =>
  getMiniCalendarDays(props.currentDate.year(), props.currentDate.month(), weekStartsOn.value)
);

const weekdays = computed(() => {
  const weekDays = ['日', '一', '二', '三', '四', '五', '六'];
  const start = weekStartsOn.value;
  const days = [];
  for (let i = 0; i < 7; i++) {
    days.push(weekDays[(start + i) % 7]);
  }
  return days;
});

const today = dayjs();

function isToday(day: number): boolean {
  return (
    props.currentDate.year() === today.year() &&
    props.currentDate.month() === today.month() &&
    day === today.date()
  );
}

function handleSelect(day: number | null) {
  if (day !== null) {
    emit('select', day);
  }
}

function handlePrev() {
  emit('prevMonth');
}

function handleNext() {
  emit('nextMonth');
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="mini-calendar fixed rounded-xl shadow-2xl p-3 z-[10000] border transition-colors duration-200"
      :class="[
        themeMode === 'dark'
          ? 'bg-gray-800/95 border-gray-600'
          : 'bg-white/95 border-gray-200'
      ]"
      :style="{
        top: position ? `${position.top}px` : '60px',
        left: position ? `${position.left}px` : '50%',
        transform: position ? 'none' : 'translateX(-50%)',
        backdropFilter: 'blur(16px)',
        WebkitBackdropFilter: 'blur(16px)'
      }"
      @mousedown.stop
      @click.stop
    >
      <!-- 月份导航 -->
      <div class="flex items-center justify-between mb-3">
        <button
          @click="handlePrev"
          class="p-1.5 rounded-lg transition-all duration-200"
          :class="[
            themeMode === 'dark'
              ? 'hover:bg-gray-700 text-gray-300 hover:text-white'
              : 'hover:bg-gray-100 text-gray-600 hover:text-gray-900'
          ]"
        >
          <ChevronLeft class="w-4 h-4" />
        </button>
        <span
          class="text-sm font-semibold"
          :class="[
            themeMode === 'dark' ? 'text-gray-100' : 'text-gray-800'
          ]"
        >
          {{ currentDate.format('YYYY年 M月') }}
        </span>
        <button
          @click="handleNext"
          class="p-1.5 rounded-lg transition-all duration-200"
          :class="[
            themeMode === 'dark'
              ? 'hover:bg-gray-700 text-gray-300 hover:text-white'
              : 'hover:bg-gray-100 text-gray-600 hover:text-gray-900'
          ]"
        >
          <ChevronRight class="w-4 h-4" />
        </button>
      </div>

      <!-- 星期标题 -->
      <div class="grid grid-cols-7 gap-1 text-center text-xs mb-2">
        <span
          v-for="(d, index) in weekdays"
          :key="d + index"
          class="w-7 font-medium"
          :class="[
            index === 0 || index === 6
              ? themeMode === 'dark'
                ? 'text-red-400'
                : 'text-red-500'
              : themeMode === 'dark'
                ? 'text-gray-400'
                : 'text-gray-500'
          ]"
        >
          {{ d }}
        </span>
      </div>

      <!-- 日期格子 -->
      <div class="grid grid-cols-7 gap-1">
        <button
          v-for="(day, index) in days"
          :key="index"
          @click="handleSelect(day)"
          :disabled="day === null"
          class="aspect-square w-7 flex items-center justify-center text-sm rounded-lg transition-all duration-200"
          :class="{
            // 当天选中
            'text-white font-semibold shadow-md': day === currentDate.date(),
            // 今天但未选中
            'font-medium': day !== currentDate.date() && isToday(day!),
            // 空白格子
            'cursor-default': day === null,
            // 普通日期
            'cursor-pointer hover:scale-105': day !== null,
            // 周末颜色
            'text-red-400': day !== null && day !== currentDate.date() && (index % 7 === 0 || index % 7 === 6),
          }"
          :style="{
            // 选中日期使用主题色背景
            backgroundColor: day === currentDate.date() ? primaryColor : undefined,
            // 今天使用主题色边框
            border: day !== currentDate.date() && isToday(day!) ? `2px solid ${primaryColor}` : '2px solid transparent',
            // 非空非选中日期的悬停背景
            '--hover-bg': themeMode === 'dark' ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.05)',
          }"
          @mouseenter="(e) => {
            if (day !== null && day !== currentDate.date()) {
              (e.target as HTMLElement).style.backgroundColor = themeMode === 'dark' ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.05)';
            }
          }"
          @mouseleave="(e) => {
            if (day !== null && day !== currentDate.date()) {
              (e.target as HTMLElement).style.backgroundColor = 'transparent';
            }
          }"
        >
          <span
            v-if="day !== null"
            :class="[
              // 非选中日期的颜色
              day !== currentDate.date()
                ? themeMode === 'dark'
                  ? isToday(day) ? '' : 'text-gray-200'
                  : isToday(day) ? '' : 'text-gray-700'
                : ''
            ]"
          >
            {{ day }}
          </span>
        </button>
      </div>
    </div>
  </Teleport>
</template>
