<script setup lang="ts">
import { ChevronLeft, ChevronRight } from 'lucide-vue-next';
import dayjs from 'dayjs';
import { getMiniCalendarDays } from '../utils/date';

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

const days = getMiniCalendarDays(props.currentDate.year(), props.currentDate.month());
const weekdays = ['日', '一', '二', '三', '四', '五', '六'];

function handleSelect(day: number) {
  emit('select', day);
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
      class="mini-calendar fixed bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-600 shadow-xl p-3 z-[10000]"
      :style="{
        top: position ? `${position.top}px` : '60px',
        left: position ? `${position.left}px` : '50%',
        transform: position ? 'none' : 'translateX(-50%)'
      }"
      @mousedown.stop
      @click.stop
    >
    <div class="flex items-center justify-between mb-2">
      <button @click="handlePrev" class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors">
        <ChevronLeft class="w-4 h-4 text-gray-700 dark:text-gray-300" />
      </button>
      <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ currentDate.format('YYYY年 M月') }}</span>
      <button @click="handleNext" class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors">
        <ChevronRight class="w-4 h-4 text-gray-700 dark:text-gray-300" />
      </button>
    </div>

    <div class="grid grid-cols-7 gap-1 text-center text-xs text-gray-500 dark:text-gray-400 mb-1">
      <span v-for="d in weekdays" :key="d">{{ d }}</span>
    </div>

    <div class="grid grid-cols-7 gap-1">
      <button
        v-for="day in days"
        :key="day"
        @click="handleSelect(day)"
        class="aspect-square w-7 flex items-center justify-center text-sm rounded-lg transition-colors"
        :class="{
          'bg-blue-500 text-white': day === currentDate.date(),
          'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-900 dark:text-gray-100': day !== currentDate.date()
        }"
      >
        {{ day }}
      </button>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.mini-calendar {
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
}
</style>
