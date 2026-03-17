<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted } from 'vue';
import { MinusCircle, Check } from 'lucide-vue-next';
import MiniCalendar from './calendar/MiniCalendar.vue';
import dayjs from 'dayjs';

const props = withDefaults(defineProps<{
  title: string;
  preview?: string;
  date?: string;
  isDone?: boolean;
  editable?: boolean;
}>(), {
  editable: true,
});

const emit = defineEmits<{
  (e: 'update:title', value: string): void;
  (e: 'update:date', value: string): void;
  (e: 'delete'): void;
  (e: 'toggleDone'): void;
  (e: 'click'): void;
}>();

const isEditing = ref(false);
const editTitle = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

// 日历相关
const showCalendar = ref(false);
const calendarPosition = ref({ top: 0, left: 0 });
const currentCalendarDate = ref(dayjs());

// 格式化日期为 mm-dd
function formatDate(dateStr?: string): string {
  if (!dateStr) return '';
  const parts = dateStr.split('-');
  if (parts.length >= 3) {
    return `${parts[1]}-${parts[2]}`;
  }
  return dateStr;
}

// 点击标题文字进入编辑
function handleTitleClick(e: Event) {
  e.stopPropagation();
  if (!props.editable) return;
  editTitle.value = props.title;
  isEditing.value = true;
  nextTick(() => {
    inputRef.value?.focus();
    inputRef.value?.select();
  });
}

// 右键标题文字触发 toggleDone
function handleTitleContextMenu(e: Event) {
  e.preventDefault();
  e.stopPropagation();
  emit('toggleDone');
}

// 保存
function handleSave() {
  const trimmed = editTitle.value.trim();
  if (trimmed && trimmed !== props.title) {
    emit('update:title', trimmed);
  }
  isEditing.value = false;
}

// 取消
function handleCancel() {
  isEditing.value = false;
}

// 失焦保存
function handleBlur() {
  if (isEditing.value) {
    handleSave();
  }
}

// 点击内容区域触发 click
function handleContentClick() {
  if (!isEditing.value) {
    emit('click');
  }
}

// 右键日期打开日历
function handleDateContextMenu(e: Event) {
  e.preventDefault();
  e.stopPropagation();
  if (!props.editable) return;

  // 计算位置
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  calendarPosition.value = {
    top: rect.bottom + 4,
    left: rect.right - 180 // 日历宽度约180px，右对齐
  };

  // 初始化日历日期
  if (props.date) {
    currentCalendarDate.value = dayjs(props.date);
  } else {
    currentCalendarDate.value = dayjs();
  }

  showCalendar.value = true;
}

// 选择日期
function handleDateSelect(day: number) {
  const newDate = currentCalendarDate.value.date(day).format('YYYY-MM-DD');
  emit('update:date', newDate);
  showCalendar.value = false;
}

// 上个月
function handlePrevMonth() {
  currentCalendarDate.value = currentCalendarDate.value.subtract(1, 'month');
}

// 下个月
function handleNextMonth() {
  currentCalendarDate.value = currentCalendarDate.value.add(1, 'month');
}

// 关闭日历
function closeCalendar() {
  showCalendar.value = false;
}

// 点击外部关闭日历
function handleClickOutside() {
  if (showCalendar.value) {
    showCalendar.value = false;
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<template>
  <div
    class="list-item group flex items-start gap-2 px-3 rounded-lg transition-colors duration-200"
    :class="preview ? 'py-2' : 'py-1.5'"
    :style="{
      backgroundColor: 'var(--theme-cell)',
      border: '1px solid var(--theme-border)',
    }"
  >
    <!-- 左侧内容 -->
    <div class="flex-1 min-w-0" @click="handleContentClick">
      <!-- 标题行 -->
      <div class="flex items-center gap-2">
        <!-- 删除按钮/点 - 固定尺寸避免抖动 -->
        <button
          @click.stop="emit('delete')"
          class="relative shrink-0 w-4 h-4 flex items-center justify-center"
          :style="{ color: 'var(--theme-text-muted)' }"
        >
          <span class="w-1.5 h-1.5 rounded-full opacity-40 group-hover:opacity-0 transition-opacity"
            :style="{ backgroundColor: 'var(--theme-text-muted)' }"></span>
          <MinusCircle class="absolute inset-0 w-4 h-4 opacity-0 group-hover:opacity-100 hover:text-red-500 dark:hover:text-red-400 transition-opacity" />
        </button>

        <!-- 只读标题 -->
        <template v-if="!isEditing">
          <span
            class="text-[13px] font-medium leading-relaxed transition-colors flex-1 truncate"
            :class="{ 'cursor-text': editable }"
            :style="{ color: isDone ? 'var(--theme-text-muted)' : 'var(--theme-text)' }"
            @click="handleTitleClick"
            @contextmenu="handleTitleContextMenu"
          >
            {{ title || '...' }}
          </span>
          <!-- 日期 -->
          <span
            v-if="date"
            class="shrink-0 text-[11px] opacity-50 cursor-pointer hover:opacity-80 transition-opacity"
            :style="{ color: 'var(--theme-text-muted)' }"
            @contextmenu.prevent.stop="handleDateContextMenu"
          >
            {{ formatDate(date) }}
          </span>
        </template>
        <!-- 编辑状态 -->
        <template v-else>
          <input
            ref="inputRef"
            v-model="editTitle"
            type="text"
            class="flex-1 min-w-0 bg-transparent outline-none p-0 text-[13px] font-medium leading-relaxed selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
            :style="{ color: 'var(--theme-text)' }"
            @blur="handleBlur"
            @keydown.enter="handleSave"
            @keydown.escape="handleCancel"
            @click.stop
          />
          <button
            @click.stop="handleSave"
            class="shrink-0 w-5 h-5 flex items-center justify-center rounded transition-all hover:bg-black/5 dark:hover:bg-white/5"
            style="color: #22c55e"
            title="保存"
          >
            <Check class="w-3.5 h-3.5" />
          </button>
        </template>
      </div>

      <!-- 预览行 -->
      <div
        v-if="preview"
        class="text-[12px] leading-relaxed truncate opacity-50 mt-0.5 pl-6"
        :style="{ color: 'var(--theme-text-muted)' }"
      >
        {{ preview }}
      </div>
    </div>

    <!-- 日历弹窗 -->
    <MiniCalendar
      :visible="showCalendar"
      :current-date="currentCalendarDate"
      :position="calendarPosition"
      @select="handleDateSelect"
      @prev-month="handlePrevMonth"
      @next-month="handleNextMonth"
      @close="closeCalendar"
    />
  </div>
</template>
