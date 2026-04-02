<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { ChevronLeft, ChevronRight } from 'lucide-vue-next';
import dayjs from 'dayjs';

const props = defineProps<{
  currentDate: dayjs.Dayjs;
  visible: boolean;
  position?: { top: number; left: number };
  centered?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:currentDate', date: dayjs.Dayjs): void;
  (e: 'select', date: dayjs.Dayjs): void;
  (e: 'close'): void;
}>();

// 直接从 localStorage 读取设置，避免依赖 useSettings 的初始化状态
function getSettings() {
  const saved = localStorage.getItem('chronos_settings');
  if (saved) {
    return JSON.parse(saved);
  }
  return null;
}

const settings = computed(() => getSettings());

const weekStartsOn = computed(() => settings.value?.week_starts_on ?? 1);
const primaryColor = computed(() => settings.value?.primary_color ?? '#3b82f6');
const themeMode = computed(() => settings.value?.theme_mode ?? 'light');
const fontFamily = computed(() => settings.value?.font_family ?? 'system-ui');
const fontSize = computed(() => settings.value?.font_size ?? 14);
const mutedTextColor = computed(() => settings.value?.muted_text_color ?? (themeMode.value === 'dark' ? '#9ca3af' : '#6b7280'));

const calendarStyle = computed(() => ({
  fontFamily: fontFamily.value,
  fontSize: `${fontSize.value}px`,
}));

const currentView = ref<'date' | 'month' | 'year'>('date');
const yearPageStart = ref(0);

// ✨ 核心修复：分离“视图日期”和“选中日期”
// 内部独立维护当前面板正在显示的年月，不污染 props.currentDate
const internalViewDate = ref(dayjs(props.currentDate));

// 当外部传入的真实日期改变时，同步更新内部视图面板
watch(() => props.currentDate, (newVal) => {
  internalViewDate.value = dayjs(newVal);
});

const displayedYears = computed(() => {
  const list = [];
  for (let i = 0; i < 12; i++) {
    list.push(yearPageStart.value + i);
  }
  return list;
});

const calendarDays = computed(() => {
  // 基于内部的 viewDate 来生成日历网格
  const current = internalViewDate.value;
  const year = current.year();
  const month = current.month();
  
  const firstDay = dayjs().year(year).month(month).date(1);
  const daysInMonth = firstDay.daysInMonth();
  
  const startDay = firstDay.day(); 
  let prefix = startDay - weekStartsOn.value;
  if (prefix < 0) prefix += 7;

  const days = [];
  const prevMonth = firstDay.subtract(1, 'month');
  const prevMonthDays = prevMonth.daysInMonth();

  for (let i = prefix - 1; i >= 0; i--) {
    days.push({
      date: prevMonthDays - i,
      isCurrentMonth: false,
      dayjsObj: prevMonth.date(prevMonthDays - i)
    });
  }

  for (let i = 1; i <= daysInMonth; i++) {
    days.push({
      date: i,
      isCurrentMonth: true,
      dayjsObj: firstDay.date(i)
    });
  }

  const nextMonth = firstDay.add(1, 'month');
  const remain = 42 - days.length;
  for (let i = 1; i <= remain; i++) {
    days.push({
      date: i,
      isCurrentMonth: false,
      dayjsObj: nextMonth.date(i)
    });
  }

  return days;
});

const weekdays = computed(() => {
  const weekDays = ['日', '一', '二', '三', '四', '五', '六'];
  const start = weekStartsOn.value;
  const d = [];
  for (let i = 0; i < 7; i++) {
    d.push(weekDays[(start + i) % 7]);
  }
  return d;
});

const weekendIndices = computed(() => {
  const start = weekStartsOn.value;
  const sundayIndex = (0 - start + 7) % 7;
  const saturdayIndex = (6 - start + 7) % 7;
  return { sundayIndex, saturdayIndex };
});

const today = dayjs();

function isToday(dayObj: typeof calendarDays.value[0]): boolean {
  return dayObj.dayjsObj.isSame(today, 'day');
}

function isWeekend(index: number): boolean {
  return index % 7 === weekendIndices.value.sundayIndex ||
         index % 7 === weekendIndices.value.saturdayIndex;
}

// 只有真正点击天数时，才触发外部更新
function handleSelect(dayObj: typeof calendarDays.value[0]) {
  emit('update:currentDate', dayObj.dayjsObj);
  emit('select', dayObj.dayjsObj);
}

// 切换月份和年份仅修改内部视图，不再触发 emit
function selectMonth(m: number) {
  internalViewDate.value = internalViewDate.value.month(m);
  currentView.value = 'date';
}

function selectYear(y: number) {
  internalViewDate.value = internalViewDate.value.year(y);
  currentView.value = 'date';
}

function handlePrevView() {
  if (currentView.value === 'date') {
    internalViewDate.value = internalViewDate.value.subtract(1, 'month');
  } else if (currentView.value === 'month') {
    internalViewDate.value = internalViewDate.value.subtract(1, 'year');
  } else if (currentView.value === 'year') {
    yearPageStart.value -= 12; 
  }
}

function handleNextView() {
  if (currentView.value === 'date') {
    internalViewDate.value = internalViewDate.value.add(1, 'month');
  } else if (currentView.value === 'month') {
    internalViewDate.value = internalViewDate.value.add(1, 'year');
  } else if (currentView.value === 'year') {
    yearPageStart.value += 12; 
  }
}

function toggleYearView() {
  if (currentView.value === 'year') {
    currentView.value = 'date';
  } else {
    currentView.value = 'year';
    const currentYear = internalViewDate.value.year();
    yearPageStart.value = Math.floor(currentYear / 10) * 10;
  }
}

function toggleMonthView() {
  currentView.value = currentView.value === 'month' ? 'date' : 'month';
}
</script>

<template>
  <Teleport to="body">
    <!-- 遮罩层 -->
    <div
      v-if="visible"
      class="fixed inset-0 z-[9999]"
      :class="themeMode === 'dark' ? 'bg-black/20' : 'bg-white/30'"
      @click="emit('close')"
    ></div>
    <!-- 日历主体 -->
    <div
      v-if="visible"
      class="mini-calendar fixed rounded-lg shadow-[0_4px_16px_rgb(0,0,0,0.12)] p-2 z-[10000] border backdrop-blur-xl w-[212px]"
      :class="[
        themeMode === 'dark'
          ? 'bg-gray-800/90 border-gray-700/60 shadow-black/50'
          : 'bg-white/90 border-gray-200/60'
      ]"
      :style="{
        top: position ? `${position.top}px` : (centered ? '50%' : '60px'),
        left: position ? `${position.left}px` : '50%',
        transform: position ? 'none' : (centered ? 'translate(-50%, -50%)' : 'translateX(-50%)'),
        '--theme-primary': primaryColor,
        ...calendarStyle,
      }"
      @mousedown.stop
      @click.stop
    >
      <div class="flex items-center justify-between mb-1.5 px-0.5">
        <button
          @click="handlePrevView"
          class="p-0.5 rounded transition-colors duration-200"
          :style="{ color: mutedTextColor }"
          :class="themeMode === 'dark' ? 'hover:bg-gray-700' : 'hover:bg-gray-100'"
        >
          <ChevronLeft class="w-4 h-4" />
        </button>

        <div class="flex items-center gap-0.5">
          <button
            @click="toggleYearView"
            class="text-xs font-bold tracking-wide px-1.5 py-0.5 rounded transition-colors"
            :class="[
              themeMode === 'dark' ? 'text-gray-200 hover:bg-gray-700' : 'text-gray-800 hover:bg-gray-100',
              currentView === 'year' ? (themeMode === 'dark' ? 'bg-gray-700' : 'bg-gray-200') : ''
            ]"
          >
            {{ internalViewDate.format('YYYY年') }}
          </button>
          <button
            @click="toggleMonthView"
            class="text-xs font-bold tracking-wide px-1.5 py-0.5 rounded transition-colors"
            :class="[
              themeMode === 'dark' ? 'text-gray-200 hover:bg-gray-700' : 'text-gray-800 hover:bg-gray-100',
              currentView === 'month' ? (themeMode === 'dark' ? 'bg-gray-700' : 'bg-gray-200') : ''
            ]"
          >
            {{ internalViewDate.format('M月') }}
          </button>
        </div>

        <button
          @click="handleNextView"
          class="p-0.5 rounded transition-colors duration-200"
          :style="{ color: mutedTextColor }"
          :class="themeMode === 'dark' ? 'hover:bg-gray-700' : 'hover:bg-gray-100'"
        >
          <ChevronRight class="w-4 h-4" />
        </button>
      </div>

      <div v-show="currentView === 'date'">
        <div class="grid grid-cols-7 gap-0.5 text-center text-xs font-bold mb-1 uppercase tracking-wider">
          <span
            v-for="(d, index) in weekdays"
            :key="d + index"
            class="w-[26px]"
            :style="{ color: isWeekend(index) ? 'var(--theme-primary)' : mutedTextColor }"
          >
            {{ d }}
          </span>
        </div>

        <div class="grid grid-cols-7 gap-0.5">
          <button
            v-for="(day, index) in calendarDays"
            :key="index"
            @click="handleSelect(day)"
            class="relative flex items-center justify-center w-[26px] h-[26px] text-xs rounded transition-all duration-200"
            :class="[
              !day.isCurrentMonth ? (themeMode === 'dark' ? 'hover:bg-gray-700/40' : 'hover:bg-gray-50') : '',
              day.isCurrentMonth && !day.dayjsObj.isSame(props.currentDate, 'day') ? (themeMode === 'dark' ? 'hover:bg-gray-700' : 'hover:bg-gray-100') : '',
              // 判断是否是真正的选中日期，必须用 props.currentDate
              day.dayjsObj.isSame(props.currentDate, 'day') ? 'font-bold' : (isToday(day) ? 'font-bold' : 'font-medium'),
            ]"
            :style="{
              backgroundColor: day.dayjsObj.isSame(props.currentDate, 'day') ? 'var(--theme-primary)' : 'transparent',
              color: day.dayjsObj.isSame(props.currentDate, 'day') 
                ? '#ffffff' 
                : !day.isCurrentMonth
                  ? (themeMode === 'dark' ? '#4b5563' : '#d1d5db')
                  : (isWeekend(index) ? 'var(--theme-primary)' : (themeMode === 'dark' ? '#e5e7eb' : '#374151')),
            }"
          >
            <span class="relative z-10">{{ day.date }}</span>
            <div 
              v-if="isToday(day) && !day.dayjsObj.isSame(props.currentDate, 'day')"
              class="absolute inset-0 rounded border opacity-50"
              :style="{ borderColor: 'var(--theme-primary)' }"
            ></div>
          </button>
        </div>
      </div>

      <div v-if="currentView === 'month'" class="h-[184px] grid grid-cols-3 gap-1.5 p-1 content-center">
        <button
          v-for="m in 12"
          :key="m"
          @click="selectMonth(m - 1)"
          class="py-2 text-xs font-bold rounded transition-colors"
          :class="[
            (m - 1) === internalViewDate.month()
              ? 'text-white'
              : (themeMode === 'dark' ? 'text-gray-300 hover:bg-gray-700' : 'text-gray-700 hover:bg-gray-100')
          ]"
          :style="{
            backgroundColor: (m - 1) === internalViewDate.month() ? 'var(--theme-primary)' : '',
          }"
        >
          {{ m }}月
        </button>
      </div>

      <div v-if="currentView === 'year'" class="h-[184px] grid grid-cols-3 gap-1.5 p-1 content-center">
        <button
          v-for="y in displayedYears"
          :key="y"
          @click="selectYear(y)"
          class="py-2 text-xs font-bold rounded transition-colors"
          :class="[
            y === internalViewDate.year()
              ? 'text-white'
              : (themeMode === 'dark' ? 'text-gray-300 hover:bg-gray-700' : 'text-gray-700 hover:bg-gray-100')
          ]"
          :style="{
            backgroundColor: y === internalViewDate.year() ? 'var(--theme-primary)' : '',
          }"
        >
          {{ y }}
        </button>
      </div>

    </div>
  </Teleport>
</template>

<style scoped>
/* 内部动画已完全移除，由父级控制 */
</style>