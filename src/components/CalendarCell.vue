<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { Check, Trash2 } from 'lucide-vue-next';
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
}>();

const isEditing = ref(false);
const editLines = ref<EditLine[]>([]);
const cellRef = ref<HTMLElement | null>(null);
const inputRefs = ref<(HTMLInputElement | null)[]>([]);
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
    const input = inputRefs.value[index];
    if (input) {
      input.focus();
      const len = input.value.length;
      input.setSelectionRange(len, len);
    }
  });
}

function toggleActiveLineDone() {
  if (activeLine.value) {
    activeLine.value.done = !activeLine.value.done;
  }
}

function handleInputKeydown(event: KeyboardEvent, index: number) {
  if (event.key === 'Escape') { saveEdit(); return; }
  if (event.key === 'Enter') {
    event.preventDefault();
    editLines.value.splice(index + 1, 0, { text: '', done: false });
    focusInput(index + 1);
  }
  if (event.key === 'Backspace' && editLines.value[index].text === '' && editLines.value.length > 1) {
    event.preventDefault();
    editLines.value.splice(index, 1);
    focusInput(Math.max(0, index - 1));
  }
  if (['ArrowUp', 'ArrowDown'].includes(event.key)) {
    event.preventDefault();
    const nextIndex = event.key === 'ArrowUp' ? index - 1 : index + 1;
    if (nextIndex >= 0 && nextIndex < editLines.value.length) focusInput(nextIndex);
  }
}
</script>

<template>
  <div ref="cellRef" class="calendar-cell relative flex flex-col transition-all duration-200 rounded-xl m-[1px]" :class="{
    'bg-[var(--cell-bg)]': isCurrentMonth && !cellStyle.backgroundColor,
    'bg-[var(--cell-bg-muted)] opacity-70': !isCurrentMonth && !cellStyle.backgroundColor,
    'ring-2 ring-inset ring-[var(--primary)] border-transparent': isToday && !isEditing,
    'editing shadow-2xl z-30 bg-white dark:bg-gray-800 scale-[1.02] !border-transparent': isEditing,
    'hover:shadow-md hover:brightness-95 dark:hover:brightness-110': !isEditing && !isLocked
  }" :style="cellStyle" @click="!isEditing && startEditing()" @contextmenu.prevent="emit('contextmenu', $event)">
    <div class="px-2 py-1 shrink-0 flex items-center justify-between select-none">
      <span class="font-semibold w-5 h-5 flex items-center justify-center rounded-full text-xs transition-colors"
        :class="isToday ? 'bg-[var(--primary)] text-white shadow-sm' : (isCurrentMonth ? 'text-[var(--text-primary)]' : 'text-[var(--text-muted)]')">
        {{ date.date() }}
      </span>
    </div>

    <div class="flex-1 overflow-hidden px-1 pb-1 flex flex-col relative">

      <div v-if="!isEditing" class="content-area h-full overflow-y-auto no-scrollbar px-1">
        <div v-for="(s, i) in schedules.filter(s => s.id !== -1 && s.content.trim() !== '')" :key="i"
          class="flex items-start gap-1 mb-0.5 text-xs leading-tight transition-all py-0.5"
          :class="s.is_done ? 'text-gray-400 line-through opacity-70' : 'text-[var(--text-primary)]'">
          <div class="shrink-0 w-1 h-1 rounded-full bg-current opacity-50 mt-1.5 mx-0.5"></div>
          <span class="truncate w-full">{{ s.content }}</span>
        </div>
      </div>

      <div v-else class="editor-container flex flex-col h-full overflow-y-auto no-scrollbar pb-6">
        <div v-for="(line, index) in editLines" :key="index"
          class="edit-line flex items-center gap-1 px-1 rounded-md group relative transition-colors"
          :class="{ 'bg-[var(--primary)]/10 dark:bg-[var(--primary)]/20': activeLineIndex === index }">
          <div class="shrink-0 w-1 h-1 rounded-full bg-gray-400 mx-0.5"></div>
          <input :ref="el => { inputRefs[index] = el as HTMLInputElement }" v-model="line.text" type="text"
            class="flex-1 bg-transparent border-none outline-none text-xs py-0.5 min-w-0"
            :class="line.done ? 'line-through text-gray-400' : 'text-[var(--text-primary)]'"
            @focus="activeLineIndex = index" @keydown="handleInputKeydown($event, index)" />
          <button @click.stop="editLines.length > 1 ? editLines.splice(index, 1) : (line.text = '')"
            class="opacity-0 group-hover:opacity-100 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 p-1 rounded transition-all">
            <Trash2 class="w-3 h-3" />
          </button>
        </div>
      </div>

      <div v-if="isEditing && activeLine" class="absolute bottom-2 right-2 z-30 flex items-center">
        <button @mousedown.prevent="toggleActiveLineDone"
          class="w-4 h-4 flex items-center justify-center transition-all cursor-pointer hover:scale-110 active:scale-95 shadow-sm"
          :class="[
            'rounded-md border',
            activeLine.done
              ? 'bg-green-500 border-green-600 shadow-green-500/20'
              : 'bg-white border-[var(--cell-border-color)] dark:bg-gray-700 dark:border-gray-500'
          ]">
          <Check v-if="activeLine.done" class="w-2.5 h-2.5 text-white" stroke-width="4" />
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