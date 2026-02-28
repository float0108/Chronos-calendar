<script setup lang="ts">
import { ChevronLeft, ChevronRight } from 'lucide-vue-next';
import dayjs from 'dayjs';
import { getMiniCalendarDays } from '../utils/date';

const props = defineProps<{
  currentDate: dayjs.Dayjs;
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', day: number): void;
  (e: 'prevMonth'): void;
  (e: 'nextMonth'): void;
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
  <div 
    v-if="visible"
    class="mini-calendar absolute top-full left-1/2 -translate-x-1/2 mt-2 w-64 bg-white rounded-xl border border-gray-200 shadow-xl p-3 z-50"
  >
    <div class="flex items-center justify-between mb-2">
      <button @click="handlePrev" class="p-1 hover:bg-gray-100 rounded transition-colors">
        <ChevronLeft class="w-4 h-4 text-gray-700" />
      </button>
      <span class="text-sm font-medium text-gray-900">{{ currentDate.format('YYYY年 M月') }}</span>
      <button @click="handleNext" class="p-1 hover:bg-gray-100 rounded transition-colors">
        <ChevronRight class="w-4 h-4 text-gray-700" />
      </button>
    </div>
    
    <div class="grid grid-cols-7 gap-1 text-center text-xs text-gray-500 mb-1">
      <span v-for="d in weekdays" :key="d">{{ d }}</span>
    </div>
    
    <div class="grid grid-cols-7 gap-1">
      <button
        v-for="day in days"
        :key="day"
        @click="handleSelect(day)"
        class="aspect-square flex items-center justify-center text-sm rounded-lg transition-colors"
        :class="{
          'bg-blue-500 text-white': day === currentDate.date(),
          'hover:bg-gray-100 text-gray-900': day !== currentDate.date()
        }"
      >
        {{ day }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.mini-calendar {
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
}
</style>
