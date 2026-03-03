<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { Check, Trash2, Palette } from 'lucide-vue-next';
import dayjs from 'dayjs';
import type { Schedule } from '../types';

interface EditLine {
  text: string;
  done: boolean;
}

const props = defineProps<{
  date: dayjs.Dayjs;
  schedules: Schedule[];
  isToday: boolean;
  isCurrentMonth: boolean;
  isLocked: boolean;
}>();

const emit = defineEmits<{
  (e: 'update', date: string, lines: EditLine[]): void;
  (e: 'navigate', direction: string): void;
  (e: 'contextmenu', event: MouseEvent): void;
  (e: 'toggleDone', schedule: Schedule): void;
}>();

const isEditing = ref(false);
const editLines = ref<EditLine[]>([]);
const cellRef = ref<HTMLElement | null>(null);
const textareaRefs = ref<(HTMLTextAreaElement | null)[]>([]);
const activeLineIndex = ref<number | null>(null);
const dateStr = props.date.format('YYYY-MM-DD');

const cellStyle = computed(() => {
  if (props.schedules.length > 0 && props.schedules[0].cell_color) {
    const color = props.schedules[0].cell_color;
    return { 
      // 1. 动态混合颜色与全局透明度变量
      // 使用 color-mix 将用户选定的颜色与透明度混合
      backgroundColor: `color-mix(in srgb, ${color} calc(var(--cell-bg-opacity, 1) * 100%), transparent)`,
      // 2. 移除 borderColor 显式设置
      // 这样它就会自动回退到 .calendar-cell 类中定义的 var(--cell-border-color)
    }; 
  }
  return {};
});

const activeLine = computed(() => {
  if (activeLineIndex.value === null || !editLines.value[activeLineIndex.value]) return null;
  return editLines.value[activeLineIndex.value];
});

function initEditLines() {
  // 过滤掉虚拟记录（id 为 -1 表示只有颜色没有内容）和空内容记录
  const validSchedules = props.schedules.filter(s => s.id !== -1 && s.content.trim() !== '');
  if (validSchedules.length === 0) return [{ text: '', done: false }];

  // 使用数据库的 is_done 字段，而不是解析 content
  return validSchedules.map(s => ({
    text: s.content.trim(),
    done: !!s.is_done
  }));
}

function handleClickOutside(event: MouseEvent) {
  if (isEditing.value && cellRef.value && !cellRef.value.contains(event.target as Node)) {
    saveEdit();
  }
}

onMounted(() => document.addEventListener('mousedown', handleClickOutside));
onUnmounted(() => document.removeEventListener('mousedown', handleClickOutside));

function startEditing() {
  if (props.isLocked) return;
  editLines.value = initEditLines();
  isEditing.value = true;
  activeLineIndex.value = editLines.value.length - 1;
  nextTick(() => focusInput(activeLineIndex.value!));
}

function saveEdit() {
  if (!isEditing.value) return;
  const validLines = editLines.value.filter(line => line.text.trim() !== '');
  emit('update', dateStr, validLines);
  isEditing.value = false;
  activeLineIndex.value = null;
}

function focusInput(index: number) {
  nextTick(() => {
    const textarea = textareaRefs.value[index];
    if (textarea) {
      textarea.focus();
      const len = textarea.value.length;
      textarea.setSelectionRange(len, len);
      autoResize(textarea);
    }
  });
}

function toggleActiveLineDone() {
  if (activeLine.value) {
    activeLine.value.done = !activeLine.value.done;
  }
}

function deleteActiveLine() {
  if (activeLineIndex.value === null) return;

  // 如果只有一个条目且为空，清空内容
  if (editLines.value.length === 1) {
    editLines.value[0].text = '';
    return;
  }

  // 如果有多个条目，删除当前条目
  const newIndex = Math.max(0, activeLineIndex.value - 1);
  editLines.value.splice(activeLineIndex.value, 1);
  activeLineIndex.value = newIndex;
  focusInput(newIndex);
}

function handleInputKeydown(event: KeyboardEvent, index: number) {
  if (event.key === 'Escape') { saveEdit(); return; }
  // Enter 创建新条目（每个换行就是一个新条目）
  if (event.key === 'Enter' && !event.shiftKey && !event.ctrlKey) {
    event.preventDefault();
    editLines.value.splice(index + 1, 0, { text: '', done: false });
    focusInput(index + 1);
    return;
  }
  if (event.key === 'Backspace' && editLines.value[index].text === '' && editLines.value.length > 1) {
    event.preventDefault();
    editLines.value.splice(index, 1);
    focusInput(Math.max(0, index - 1));
  }
  // 上下箭头导航
  if (['ArrowUp', 'ArrowDown'].includes(event.key)) {
    const textarea = textareaRefs.value[index];
    if (!textarea) return;
    const cursorPos = textarea.selectionStart;
    const text = textarea.value;

    // ArrowUp: 只在光标在开头时导航到上一条目
    if (event.key === 'ArrowUp' && index > 0 && cursorPos === 0) {
      event.preventDefault();
      focusInput(index - 1);
    }
    // ArrowDown: 只在光标在末尾时导航到下一条目
    if (event.key === 'ArrowDown' && index < editLines.value.length - 1 && cursorPos === text.length) {
      event.preventDefault();
      focusInput(index + 1);
    }
  }
}

function autoResize(textarea: HTMLTextAreaElement) {
  textarea.style.height = 'auto';
  textarea.style.height = textarea.scrollHeight + 'px';
}

function handleScheduleContextMenu(event: MouseEvent, schedule: Schedule) {
  event.preventDefault();
  event.stopPropagation();
  emit('toggleDone', schedule);
}

function handleColorButtonClick(event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  emit('contextmenu', event);
}

function handleEditLineContextMenu(event: MouseEvent, index: number) {
  event.preventDefault();
  event.stopPropagation();
  // 切换该行的完成状态
  editLines.value[index].done = !editLines.value[index].done;
}
</script>

<template>
  <div ref="cellRef" class="calendar-cell relative flex flex-col transition-all duration-200 rounded-xl m-[1px]" :class="{
    'bg-[var(--cell-bg)]': isCurrentMonth && !cellStyle.backgroundColor,
    'bg-[var(--cell-bg-muted)] opacity-70': !isCurrentMonth && !cellStyle.backgroundColor,
    'ring-2 ring-inset ring-[var(--primary)] border-transparent': isToday && !isEditing,
    'editing shadow-2xl z-30 bg-white dark:bg-gray-800 scale-[1.02] !border-transparent': isEditing,
    'hover:shadow-md hover:brightness-95 dark:hover:brightness-110': !isEditing && !isLocked
  }" :style="cellStyle" @click="!isEditing && startEditing()">
    <div class="px-2 py-1 shrink-0 flex items-center justify-between select-none">
      <span class="font-semibold w-5 h-5 flex items-center justify-center rounded-full text-xs transition-colors"
        :class="isToday ? 'bg-[var(--primary)] text-white shadow-sm' : (isCurrentMonth ? 'text-[var(--text-primary)]' : 'text-[var(--text-muted)]')">
        {{ date.date() }}
      </span>
    </div>

    <div class="flex-1 overflow-hidden px-1 pb-1 flex flex-col relative">

      <div v-if="!isEditing" class="content-area h-full overflow-y-auto no-scrollbar px-1">
        <div v-for="(s, i) in schedules.filter(s => s.id !== -1 && s.content.trim() !== '')" :key="i"
          class="flex items-center gap-1 mb-0.5 text-xs leading-tight transition-all py-0.5 cursor-pointer"
          :class="s.is_done ? 'text-gray-400 line-through opacity-70' : 'text-[var(--text-primary)]'"
          @contextmenu.prevent="handleScheduleContextMenu($event, s)">
          <div class="shrink-0 w-1 h-1 rounded-full bg-current opacity-50"></div>
          <span class="break-words flex-1">{{ s.content }}</span>
        </div>
      </div>

      <div v-else class="editor-container flex flex-col h-full overflow-y-auto no-scrollbar pb-6">
        <div v-for="(line, index) in editLines" :key="index"
          class="edit-line flex items-start gap-1 px-1 rounded-md group relative transition-colors"
          :class="{ 'bg-[var(--primary)]/10 dark:bg-[var(--primary)]/20': activeLineIndex === index }"
          @contextmenu.prevent="handleEditLineContextMenu($event, index)">
          <div class="shrink-0 w-1 h-1 rounded-full bg-gray-400 mt-1.5"></div>
          <textarea :ref="el => { textareaRefs[index] = el as HTMLTextAreaElement }" v-model="line.text"
            rows="1"
            class="flex-1 bg-transparent border-none outline-none text-xs py-0.5 min-w-0 resize-none overflow-hidden leading-tight break-words"
            :class="line.done ? 'line-through text-gray-400' : 'text-[var(--text-primary)]'"
            @focus="activeLineIndex = index"
            @keydown="handleInputKeydown($event, index)"
            @input="autoResize($event.target as HTMLTextAreaElement)"></textarea>
        </div>
      </div>

      <!-- 左下角垃圾桶：删除当前条目 -->
      <div v-if="isEditing && activeLineIndex !== null" class="absolute bottom-2 left-2 z-30">
        <button @mousedown.prevent="deleteActiveLine"
          class="w-6 h-6 flex items-center justify-center rounded-md bg-red-50 hover:bg-red-100 text-red-500 transition-all hover:scale-110 active:scale-95 shadow-sm dark:bg-red-900/20 dark:hover:bg-red-900/30">
          <Trash2 class="w-3 h-3" />
        </button>
      </div>

      <!-- 右下角调色盘：单元格颜色 -->
      <div v-if="isEditing" class="absolute bottom-2 right-2 z-30">
        <button @mousedown.prevent="handleColorButtonClick"
          class="w-6 h-6 flex items-center justify-center rounded-md bg-white hover:bg-gray-50 text-gray-600 transition-all hover:scale-110 active:scale-95 shadow-sm border border-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-300 dark:border-gray-600">
          <Palette class="w-3 h-3" />
        </button>
      </div>

    </div>
  </div>
</template>

<style scoped>
.no-scrollbar::-webkit-scrollbar {
  display: none;
}

.no-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.calendar-cell {
  height: 100%;
  border-width: var(--cell-border-width, 1px);
  border-style: solid;
  border-color: var(--cell-border-color, var(--border-light, #e5e7eb));
  cursor: pointer;
}

.edit-line input:focus {
  box-shadow: none;
}
</style>