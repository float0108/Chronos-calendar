<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { Trash2, Palette } from 'lucide-vue-next';
import dayjs from 'dayjs';
import type { Schedule } from '../types';

interface EditLine {
  text: string;
  done: boolean;
}

interface EditHistory {
  lines: EditLine[];
  activeLineIndex: number | null;
  cursorPosition: number;
}

const props = defineProps<{
  date: dayjs.Dayjs;
  schedules: Schedule[];
  isToday: boolean;
  isCurrentMonth: boolean;
  isLocked: boolean;
  viewMode?: 'todo' | 'done';
}>();

const emit = defineEmits<{
  (e: 'update', date: string, lines: EditLine[]): void;
  (e: 'navigate', direction: string): void;
  (e: 'contextmenu', event: MouseEvent): void;
  (e: 'toggleDone', schedule: Schedule): void;
  (e: 'editDescription', schedule: Schedule): void;
}>();

const isEditing = ref(false);
const editLines = ref<EditLine[]>([]);
const cellRef = ref<HTMLElement | null>(null);
const textareaRefs = ref<(HTMLTextAreaElement | null)[]>([]);
const activeLineIndex = ref<number | null>(null);
const dateStr = props.date.format('YYYY-MM-DD');

// 悬停描述浮窗状态
const hoveredSchedule = ref<Schedule | null>(null);
const tooltipPosition = ref({ x: 0, y: 0 });
let tooltipTimeout: number | null = null;
let animationFrameId: number | null = null;
let pendingPosition = { x: 0, y: 0 };

// 编辑历史管理（单元格级别的撤销/重做系统）
// 注意：这是单元格编辑时的局部撤销/重做，独立于 App.vue 中的全局撤销/重做系统
// - 单元格级别：仅在编辑时有效，撤销/重做文本编辑操作
// - 全局级别：在任何时候有效，撤销/重做完成状态切换、日程编辑等操作
const undoHistory = ref<EditHistory[]>([]);
const redoHistory = ref<EditHistory[]>([]);
const maxHistory = 50;
let lastSaveTime = 0;
const saveDelay = 500; // 500ms 防抖

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

// 保存当前编辑状态到历史
function saveHistory() {
  const textarea = activeLineIndex.value !== null ? textareaRefs.value[activeLineIndex.value] : null;
  const cursorPosition = textarea ? textarea.selectionStart : 0;

  undoHistory.value.push({
    lines: JSON.parse(JSON.stringify(editLines.value)),
    activeLineIndex: activeLineIndex.value,
    cursorPosition,
  });

  if (undoHistory.value.length > maxHistory) {
    undoHistory.value.shift();
  }

  // 清空重做历史
  redoHistory.value = [];
}

// 防抖保存历史（用于文本输入）
function saveHistoryDebounced() {
  const now = Date.now();
  if (now - lastSaveTime > saveDelay) {
    saveHistory();
    lastSaveTime = now;
  }
}

// 撤销
function handleEditUndo() {
  if (undoHistory.value.length === 0) return;

  const textarea = activeLineIndex.value !== null ? textareaRefs.value[activeLineIndex.value] : null;
  const cursorPosition = textarea ? textarea.selectionStart : 0;

  // 保存当前状态到重做历史
  redoHistory.value.push({
    lines: JSON.parse(JSON.stringify(editLines.value)),
    activeLineIndex: activeLineIndex.value,
    cursorPosition,
  });

  // 恢复上一个状态
  const history = undoHistory.value.pop()!;
  editLines.value = history.lines;
  activeLineIndex.value = history.activeLineIndex;

  // 恢复光标位置
  nextTick(() => {
    if (history.activeLineIndex !== null) {
      const textarea = textareaRefs.value[history.activeLineIndex];
      if (textarea) {
        textarea.focus();
        textarea.setSelectionRange(history.cursorPosition, history.cursorPosition);
      }
    }
  });
}

// 重做
function handleEditRedo() {
  if (redoHistory.value.length === 0) return;

  const textarea = activeLineIndex.value !== null ? textareaRefs.value[activeLineIndex.value] : null;
  const cursorPosition = textarea ? textarea.selectionStart : 0;

  // 保存当前状态到撤销历史
  undoHistory.value.push({
    lines: JSON.parse(JSON.stringify(editLines.value)),
    activeLineIndex: activeLineIndex.value,
    cursorPosition,
  });

  // 恢复下一个状态
  const history = redoHistory.value.pop()!;
  editLines.value = history.lines;
  activeLineIndex.value = history.activeLineIndex;

  // 恢复光标位置
  nextTick(() => {
    if (history.activeLineIndex !== null) {
      const textarea = textareaRefs.value[history.activeLineIndex];
      if (textarea) {
        textarea.focus();
        textarea.setSelectionRange(history.cursorPosition, history.cursorPosition);
      }
    }
  });
}

function handleClickOutside(event: MouseEvent) {
  if (isEditing.value && cellRef.value && !cellRef.value.contains(event.target as Node)) {
    saveEdit();
  }
}

onMounted(() => document.addEventListener('mousedown', handleClickOutside));
onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
  if (tooltipTimeout) {
    clearTimeout(tooltipTimeout);
  }
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
  }
});

function startEditing() {
  if (props.isLocked) return;
  editLines.value = initEditLines();
  isEditing.value = true;
  activeLineIndex.value = editLines.value.length - 1;

  // 初始化历史
  undoHistory.value = [];
  redoHistory.value = [];

  nextTick(() => {
    focusInput(activeLineIndex.value!);
    // 初始化所有 textarea 的高度
    textareaRefs.value.forEach(textarea => {
      if (textarea) {
        autoResize(textarea);
      }
    });
  });
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
    saveHistory();
    activeLine.value.done = !activeLine.value.done;
  }
}

// 将来可能使用，暂时保留
void toggleActiveLineDone;

function deleteActiveLine() {
  if (activeLineIndex.value === null) return;

  // 如果只有一个条目且为空，清空内容
  if (editLines.value.length === 1) {
    if (editLines.value[0].text !== '') {
      saveHistory();
      editLines.value[0].text = '';
    }
    return;
  }

  // 如果有多个条目，删除当前条目
  saveHistory();
  const newIndex = Math.max(0, activeLineIndex.value - 1);
  editLines.value.splice(activeLineIndex.value, 1);
  activeLineIndex.value = newIndex;
  focusInput(newIndex);
}

function handleInputKeydown(event: KeyboardEvent, index: number) {
  // Ctrl+Z 撤销
  if ((event.ctrlKey || event.metaKey) && event.key === 'z' && !event.shiftKey) {
    event.preventDefault();
    handleEditUndo();
    return;
  }

  // Ctrl+Y 重做
  if ((event.ctrlKey || event.metaKey) && event.key === 'y') {
    event.preventDefault();
    handleEditRedo();
    return;
  }

  // Ctrl+Shift+Z 也支持重做
  if ((event.ctrlKey || event.metaKey) && event.key === 'z' && event.shiftKey) {
    event.preventDefault();
    handleEditRedo();
    return;
  }

  if (event.key === 'Escape') { saveEdit(); return; }
  // Enter 创建新条目（每个换行就是一个新条目）
  if (event.key === 'Enter' && !event.shiftKey && !event.ctrlKey) {
    event.preventDefault();
    saveHistory();
    editLines.value.splice(index + 1, 0, { text: '', done: false });
    focusInput(index + 1);
    return;
  }
  if (event.key === 'Backspace' && editLines.value[index].text === '' && editLines.value.length > 1) {
    event.preventDefault();
    saveHistory();
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

function handleInput(event: Event, _index: number) {
  const textarea = event.target as HTMLTextAreaElement;
  autoResize(textarea);
  saveHistoryDebounced();
}

function handleScheduleContextMenu(event: MouseEvent, schedule: Schedule) {
  event.preventDefault();
  event.stopPropagation();
  emit('toggleDone', schedule);
}

function handleScheduleMiddleClick(event: MouseEvent, schedule: Schedule) {
  if (event.button === 1) {
    event.preventDefault();
    event.stopPropagation();
    emit('editDescription', schedule);
  }
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
  saveHistory();
  editLines.value[index].done = !editLines.value[index].done;
}

function handleScheduleMouseEnter(event: MouseEvent, schedule: Schedule) {
  if (!schedule.description) return;

  // 保存初始位置
  pendingPosition = { x: event.clientX + 10, y: event.clientY + 10 };

  tooltipTimeout = window.setTimeout(() => {
    hoveredSchedule.value = schedule;
    tooltipPosition.value = { ...pendingPosition };
  }, 500); // 500ms 延迟显示
}

function handleScheduleMouseLeave() {
  if (tooltipTimeout) {
    clearTimeout(tooltipTimeout);
    tooltipTimeout = null;
  }
  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
    animationFrameId = null;
  }
  hoveredSchedule.value = null;
}

function handleScheduleMouseMove(event: MouseEvent) {
  // 更新待处理位置
  pendingPosition = { x: event.clientX + 10, y: event.clientY + 10 };

  // 如果浮窗已显示，使用 requestAnimationFrame 更新位置
  if (hoveredSchedule.value && !animationFrameId) {
    animationFrameId = requestAnimationFrame(() => {
      tooltipPosition.value = { ...pendingPosition };
      animationFrameId = null;
    });
  }
}
</script>

<template>
  <div ref="cellRef" class="calendar-cell relative flex flex-col transition-all duration-200 rounded-xl" :class="{
    'bg-[var(--cell-bg)]': isCurrentMonth && !cellStyle.backgroundColor,
    'bg-[var(--cell-bg-muted)]': !isCurrentMonth && !cellStyle.backgroundColor,
    'today-cell': isToday && !isEditing,
    'editing shadow-2xl z-30 bg-white dark:bg-gray-800 scale-[1.02] !border-transparent': isEditing,
    'hover:shadow-md hover:brightness-95 dark:hover:brightness-110': !isEditing && !isLocked
  }" :style="cellStyle" @click="!isEditing && startEditing()">
    <div class="px-2 py-1 shrink-0 flex items-center justify-between select-none">
      <span class="font-semibold w-5 h-5 flex items-center justify-center rounded-full text-xs transition-colors text-[var(--text-primary)]">
        {{ date.date() }}
      </span>
    </div>

    <div class="flex-1 overflow-hidden px-1 pb-1 flex flex-col relative">

      <div v-if="!isEditing" class="content-area h-full overflow-y-auto no-scrollbar px-1">
        <div v-for="(s, i) in schedules.filter(s => s.id !== -1 && s.content.trim() !== '')" :key="i"
          class="flex items-center gap-1 mb-0.5 text-xs leading-tight transition-all py-0.5"
          :class="(s.is_done && viewMode !== 'done') ? 'text-gray-500 dark:text-gray-400 line-through opacity-90' : 'text-[var(--text-primary)]'"
          @contextmenu.prevent="handleScheduleContextMenu($event, s)"
          @mousedown.prevent="handleScheduleMiddleClick($event, s)"
          @mouseenter="handleScheduleMouseEnter($event, s)"
          @mouseleave="handleScheduleMouseLeave"
          @mousemove="handleScheduleMouseMove">
          <div class="shrink-0 w-1 h-1 rounded-full bg-current opacity-50"></div>
          <span class="content-text flex-1">{{ s.content }}</span>
        </div>
      </div>

      <div v-else class="editor-container flex flex-col h-full overflow-y-auto no-scrollbar pb-6">
        <div v-for="(line, index) in editLines" :key="index"
          class="edit-line flex items-start gap-1 mb-1 rounded-md group relative transition-all py-0.5"
          :class="{ 'bg-[var(--primary)]/10 dark:bg-[var(--primary)]/20': activeLineIndex === index }"
          @contextmenu.prevent="handleEditLineContextMenu($event, index)">
          <div class="shrink-0 w-1 h-1 rounded-full bg-current opacity-50 mt-1.5"></div>
          <textarea :ref="el => { textareaRefs[index] = el as HTMLTextAreaElement }" v-model="line.text"
            rows="1"
            class="flex-1 bg-transparent border-none outline-none text-xs min-w-0 resize-none overflow-hidden leading-tight textarea-wrap"
            :class="line.done ? 'line-through text-gray-400 opacity-70' : 'text-[var(--text-primary)]'"
            @focus="activeLineIndex = index"
            @keydown="handleInputKeydown($event, index)"
            @input="handleInput($event, index)"></textarea>
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

    <!-- 描述浮窗 -->
    <Teleport to="body">
      <div
        v-if="hoveredSchedule && hoveredSchedule.description"
        class="fixed z-[9999] max-w-xs px-3 py-2 text-xs bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg pointer-events-none"
        :style="{
          left: tooltipPosition.x + 'px',
          top: tooltipPosition.y + 'px'
        }"
      >
        <div class="font-medium text-gray-900 dark:text-gray-100 mb-1">{{ hoveredSchedule.content }}</div>
        <div class="text-gray-600 dark:text-gray-400 whitespace-pre-wrap">{{ hoveredSchedule.description }}</div>
      </div>
    </Teleport>
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
}

.today-cell {
  outline-style: solid;
  outline-width: var(--cell-border-width, 2px);
  outline-offset: calc(var(--cell-border-width, 2px) * -1);
  outline-color: var(--primary);
}

.edit-line input:focus {
  box-shadow: none;
}

.content-text {
  word-wrap: break-word;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.textarea-wrap {
  word-wrap: break-word;
  overflow-wrap: anywhere;
  word-break: break-word;
}
</style>