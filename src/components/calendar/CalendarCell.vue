<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { Palette, Pencil, Check, MinusCircle } from 'lucide-vue-next';
import dayjs from 'dayjs';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { Schedule } from '../../types';
import { useEditHistory, type EditLine } from '../../composables/useEditHistory';
import ScheduleTooltip from '../ui/ScheduleTooltip.vue';

const props = defineProps<{
  date: dayjs.Dayjs;
  schedules: Schedule[];
  isToday: boolean;
  isCurrentMonth: boolean;
  isLocked: boolean;
  viewMode?: 'todo' | 'done';
}>();

const emit = defineEmits<{
  (e: 'update', date: string, lines: EditLine[], viewMode?: 'todo' | 'done'): void;
  (e: 'navigate', direction: string): void;
  (e: 'contextmenu', event: MouseEvent): void;
  (e: 'toggleDone', schedule: Schedule): void;
  (e: 'editDescription', schedule: Schedule): void;
  (e: 'scheduleDrop', date: string, event: DragEvent): void;
}>();

const { saveHistory, saveHistoryDebounced, handleEditUndo, handleEditRedo, resetHistory } = useEditHistory();

const isEditing = ref(false);
const editLines = ref<EditLine[]>([]);
const cellRef = ref<HTMLElement | null>(null);
const textareaRefs = ref<(HTMLTextAreaElement | null)[]>([]);
const tooltipRef = ref<InstanceType<typeof ScheduleTooltip> | null>(null);
const activeLineIndex = ref<number | null>(null);
const dateStr = props.date.format('YYYY-MM-DD');

// 悬停描述浮窗状态
const hoveredSchedule = ref<Schedule | null>(null);
const tooltipPosition = ref({ x: 0, y: 0 });
let tooltipTimeout: number | null = null;
let unlistenFocus: (() => void) | null = null;

const cellStyle = computed(() => {
  if (props.schedules.length > 0 && props.schedules[0].cell_color) {
    const color = props.schedules[0].cell_color;
    return {
      backgroundColor: `color-mix(in srgb, ${color} calc(var(--cell-bg-opacity, 1) * 100%), transparent)`,
    };
  }
  return {};
});

const borderStyleClass = computed(() => {
  const style = getComputedStyle(document.documentElement).getPropertyValue('--cell-border-style').trim() || 'solid';
  if (style === 'solid') return '';
  return `border-style-${style}`;
});

function initEditLines(): EditLine[] {
  const validSchedules = props.schedules.filter(s => s.id !== -1 && s.content.trim() !== '');
  if (validSchedules.length === 0) return [{ text: '', done: false }];

  return validSchedules.map(s => ({
    id: s.id,
    text: s.content.trim(),
    done: !!s.is_done
  }));
}

function handleClickOutside(event: MouseEvent) {
  if (isEditing.value && cellRef.value && !cellRef.value.contains(event.target as Node)) {
    saveEdit();
  }
}

function handleGlobalClick(event: MouseEvent) {
  if (hoveredSchedule.value) {
    const target = event.target as HTMLElement;
    if (!target.closest('.schedule-item')) {
      hoveredSchedule.value = null;
      if (tooltipTimeout) {
        clearTimeout(tooltipTimeout);
        tooltipTimeout = null;
      }
    }
  }
}

onMounted(async () => {
  document.addEventListener('mousedown', handleClickOutside);
  document.addEventListener('click', handleGlobalClick);
  try {
    unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload }) => {
      if (!payload && isEditing.value) {
        saveEdit();
      }
    });
  } catch (error) {
    console.error('Failed to listen focus change:', error);
  }
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
  document.removeEventListener('click', handleGlobalClick);
  if (tooltipTimeout) {
    clearTimeout(tooltipTimeout);
  }
  if (dragOverTimeout.value) {
    clearTimeout(dragOverTimeout.value);
  }
  if (unlistenFocus) {
    unlistenFocus();
  }
});

function startEditing() {
  if (props.isLocked) return;

  hoveredSchedule.value = null;
  if (tooltipTimeout) {
    clearTimeout(tooltipTimeout);
    tooltipTimeout = null;
  }

  const lines = initEditLines();
  editLines.value = lines;
  isEditing.value = true;
  activeLineIndex.value = editLines.value.length - 1;

  resetHistory();

  nextTick(() => {
    focusInput(activeLineIndex.value!);
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
  emit('update', dateStr, validLines, props.viewMode);
  isEditing.value = false;
  activeLineIndex.value = null;
}

function cancelEdit() {
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

function handleInputKeydown(event: KeyboardEvent, index: number) {
  // Ctrl+Z 撤销
  if ((event.ctrlKey || event.metaKey) && event.key === 'z' && !event.shiftKey) {
    event.preventDefault();
    handleEditUndo(
      editLines.value,
      activeLineIndex.value,
      textareaRefs.value,
      (history) => {
        editLines.value = history.lines;
        activeLineIndex.value = history.activeLineIndex;
      }
    );
    return;
  }

  // Ctrl+Y 重做
  if ((event.ctrlKey || event.metaKey) && event.key === 'y') {
    event.preventDefault();
    handleEditRedo(
      editLines.value,
      activeLineIndex.value,
      textareaRefs.value,
      (history) => {
        editLines.value = history.lines;
        activeLineIndex.value = history.activeLineIndex;
      }
    );
    return;
  }

  // Ctrl+Shift+Z 也支持重做
  if ((event.ctrlKey || event.metaKey) && event.key === 'z' && event.shiftKey) {
    event.preventDefault();
    handleEditRedo(
      editLines.value,
      activeLineIndex.value,
      textareaRefs.value,
      (history) => {
        editLines.value = history.lines;
        activeLineIndex.value = history.activeLineIndex;
      }
    );
    return;
  }

  // Ctrl+S 保存
  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault();
    saveEdit();
    return;
  }

  if (event.key === 'Escape') {
    cancelEdit();
    return;
  }

  // Enter 创建新条目
  if (event.key === 'Enter' && !event.shiftKey && !event.ctrlKey) {
    event.preventDefault();
    saveHistory(editLines.value, activeLineIndex.value, textareaRefs.value);
    editLines.value.splice(index + 1, 0, { text: '', done: false });
    focusInput(index + 1);
    return;
  }

  if (event.key === 'Backspace' && editLines.value[index].text === '' && editLines.value.length > 1) {
    event.preventDefault();
    saveHistory(editLines.value, activeLineIndex.value, textareaRefs.value);
    editLines.value.splice(index, 1);
    focusInput(Math.max(0, index - 1));
  }

  // 上下箭头导航
  if (['ArrowUp', 'ArrowDown'].includes(event.key)) {
    const textarea = textareaRefs.value[index];
    if (!textarea) return;
    const cursorPos = textarea.selectionStart;
    const text = textarea.value;

    if (event.key === 'ArrowUp' && index > 0 && cursorPos === 0) {
      event.preventDefault();
      focusInput(index - 1);
    }
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
  saveHistoryDebounced(editLines.value, activeLineIndex.value, textareaRefs.value);
}

function handleScheduleClick(event: MouseEvent, schedule: Schedule) {
  event.stopPropagation();
  emit('editDescription', schedule);
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
  saveHistory(editLines.value, activeLineIndex.value, textareaRefs.value);
  editLines.value[index].done = !editLines.value[index].done;
}

function handleScheduleMouseEnter(event: MouseEvent, schedule: Schedule) {
  if (!schedule.description) return;

  if (tooltipTimeout) {
    clearTimeout(tooltipTimeout);
  }

  const rect = (event.target as HTMLElement).getBoundingClientRect();
  // 预览窗口宽度为窗口宽度的 1/5（约 1.5 个 cell 宽度，假设 7 列时 cell 宽度约为窗口宽度/7）
  const tooltipWidth = window.innerWidth / 5;
  const tooltipHeight = 192;
  const gap = 10;

  // 计算左右两侧可用空间
  const rightSpace = window.innerWidth - rect.right - gap;
  const leftSpace = rect.left - gap;

  // 计算水平位置
  let x: number;
  if (rightSpace >= tooltipWidth) {
    x = rect.right + gap;
  } else if (leftSpace >= tooltipWidth) {
    x = rect.left - gap - tooltipWidth;
  } else {
    // 两侧都不够，选择空间更大的一侧
    if (rightSpace >= leftSpace) {
      x = Math.min(rect.right + gap, window.innerWidth - tooltipWidth - gap);
    } else {
      x = Math.max(rect.left - gap - tooltipWidth, gap);
    }
  }

  // 计算垂直位置，防止底部溢出
  let y = rect.top;
  if (y + tooltipHeight > window.innerHeight) {
    y = window.innerHeight - tooltipHeight - gap;
    if (y < 0) y = gap;
  }

  tooltipPosition.value = { x, y };

  tooltipTimeout = window.setTimeout(() => {
    hoveredSchedule.value = schedule;
  }, 500);
}

function handleScheduleMouseLeave() {
  if (tooltipTimeout) {
    clearTimeout(tooltipTimeout);
    tooltipTimeout = null;
  }
  hoveredSchedule.value = null;
}

function handleScheduleWheel(event: WheelEvent) {
  if (hoveredSchedule.value && tooltipRef.value) {
    tooltipRef.value.scrollBy(event.deltaY);
  }
}

// 拖拽相关状态
const isDragOver = ref(false);
const dragOverTimeout = ref<number | null>(null);

function handleDragStart(event: DragEvent, schedule: Schedule) {
  if (props.isLocked) {
    event.preventDefault();
    return;
  }

  if (!event.dataTransfer) return;

  // 设置拖拽数据
  event.dataTransfer.setData('application/json', JSON.stringify({
    id: schedule.id,
    content: schedule.content,
    is_done: schedule.is_done,
    create_date: schedule.create_date,
    done_date: schedule.done_date,
    sourceDate: dateStr,
    viewMode: props.viewMode
  }));

  event.dataTransfer.effectAllowed = 'move';
  event.dataTransfer.dropEffect = 'move';

  // 添加拖拽样式
  const target = event.target as HTMLElement;
  target.style.opacity = '0.5';
}

function handleDragEnd(event: DragEvent) {
  const target = event.target as HTMLElement;
  target.style.opacity = '1';
}

function handleDragOver(event: DragEvent) {
  event.preventDefault();
  if (!event.dataTransfer) return;

  // 只接受来自日程条目的拖拽
  if (event.dataTransfer.types.includes('application/json')) {
    event.dataTransfer.dropEffect = 'move';

    // 添加拖拽悬停效果
    if (!isDragOver.value) {
      isDragOver.value = true;
    }

    // 清除之前的超时
    if (dragOverTimeout.value) {
      clearTimeout(dragOverTimeout.value);
    }

    // 设置超时以在拖拽离开时移除效果
    dragOverTimeout.value = window.setTimeout(() => {
      isDragOver.value = false;
    }, 100);
  }
}

function handleDragLeave(event: DragEvent) {
  // 检查是否真的离开了单元格
  const cell = cellRef.value;
  if (cell && !cell.contains(event.relatedTarget as Node)) {
    isDragOver.value = false;
    if (dragOverTimeout.value) {
      clearTimeout(dragOverTimeout.value);
      dragOverTimeout.value = null;
    }
  }
}

function handleDrop(event: DragEvent) {
  event.preventDefault();
  isDragOver.value = false;

  if (dragOverTimeout.value) {
    clearTimeout(dragOverTimeout.value);
    dragOverTimeout.value = null;
  }

  emit('scheduleDrop', dateStr, event);
}
</script>

<template>
  <div ref="cellRef" class="calendar-cell relative flex flex-col transition-all duration-200 rounded-xl" :class="{
    'bg-[var(--cell-bg)]': isCurrentMonth && !cellStyle.backgroundColor,
    'bg-[var(--cell-bg-muted)]': !isCurrentMonth && !cellStyle.backgroundColor,
    'today-cell': isToday && !isEditing,
    'editing shadow-2xl z-30 bg-[var(--cell-bg)] scale-[1.02] !border-transparent': isEditing,
    'drag-over': isDragOver,
    [borderStyleClass]: !!borderStyleClass
  }" :style="cellStyle"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop">
    <div class="px-2 py-1 shrink-0 flex items-center justify-between select-none">
      <span class="font-semibold w-5 h-5 flex items-center justify-center rounded-full text-xs transition-colors text-[var(--text-primary)]">
        {{ date.date() }}
      </span>
      <!-- 编辑/保存按钮 -->
      <button v-if="!isLocked"
        @click.stop="isEditing ? saveEdit() : startEditing()"
        class="edit-btn w-5 h-5 flex items-center justify-center rounded opacity-0 hover:bg-black/5 dark:hover:bg-white/10 transition-all"
        :style="{ color: 'var(--text-muted)' }">
        <Check v-if="isEditing" class="w-3.5 h-3.5" />
        <Pencil v-else class="w-3 h-3" />
      </button>
    </div>

    <div class="flex-1 overflow-hidden px-1 pb-1 flex flex-col relative">
      <div v-if="!isEditing" class="content-area h-full overflow-y-auto no-scrollbar px-1">
        <div v-for="(s, i) in schedules.filter(s => s.id !== -1 && s.content.trim() !== '')" :key="i"
          class="schedule-item flex items-center gap-1 mb-0.5 text-xs leading-tight transition-all py-0.5 px-1 -mx-1 rounded"
          :class="[
            (s.is_done && viewMode !== 'done') ? 'opacity-90' : '',
            { 'active:cursor-grabbing': !isLocked }
          ]"
          :style="{ color: (s.is_done && viewMode !== 'done') ? 'var(--text-muted)' : 'var(--text-primary)' }""
          :draggable="!isLocked"
          @click="handleScheduleClick($event, s)"
          @contextmenu.prevent="handleScheduleContextMenu($event, s)"
          @mouseenter="handleScheduleMouseEnter($event, s)"
          @mouseleave="handleScheduleMouseLeave"
          @wheel="handleScheduleWheel"
          @dragstart="handleDragStart($event, s)"
          @dragend="handleDragEnd">
          <button
            @click.stop="emit('toggleDone', s)"
            class="relative shrink-0 w-3 h-3 flex items-center justify-center marker-btn"
            :style="{ color: 'var(--text-muted)' }"
          >
            <span class="w-1 h-1 rounded-full opacity-50 marker-dot"
              :style="{ backgroundColor: 'currentColor' }"></span>
            <MinusCircle class="absolute inset-0 w-3 h-3 opacity-0 marker-delete hover:text-red-500 dark:hover:text-red-400" />
          </button>
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
            :class="line.done ? 'line-through opacity-70' : ''"
            :style="{ color: line.done ? 'var(--text-muted)' : 'var(--text-primary)' }"
            @focus="activeLineIndex = index"
            @keydown="handleInputKeydown($event, index)"
            @input="handleInput($event, index)"></textarea>
        </div>
      </div>

<!-- 右下角调色盘：单元格颜色 -->
      <div v-if="isEditing" class="absolute bottom-2 right-2 z-30">
        <button @mousedown.prevent="handleColorButtonClick"
          class="w-6 h-6 flex items-center justify-center rounded-md transition-all hover:scale-110 active:scale-95 shadow-sm"
          :style="{
            backgroundColor: 'var(--theme-cell)',
            color: 'var(--text-muted)',
            border: '1px solid var(--theme-border)'
          }">
          <Palette class="w-3 h-3" />
        </button>
      </div>
    </div>

    <!-- 描述浮窗 -->
    <ScheduleTooltip
      ref="tooltipRef"
      :schedule="hoveredSchedule"
      :position="tooltipPosition"
    />
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
  border-color: var(--cell-border-color, var(--border-light, #e5e7eb));
}

/* 默认实线 */
.calendar-cell:not(.border-style-dashed):not(.border-style-dotted):not(.border-style-dash-dot):not(.border-style-dash-dot-dot) {
  border-style: solid;
}

/* 虚线 */
.calendar-cell.border-style-dashed {
  border-style: dashed;
}

/* 点线 */
.calendar-cell.border-style-dotted {
  border-style: dotted;
}

/* 点划线 - 使用 border-image 模拟，注意不支持圆角 */
.calendar-cell.border-style-dash-dot {
  border-style: solid;
  border-image: repeating-linear-gradient(
    90deg,
    var(--cell-border-color) 0px,
    var(--cell-border-color) calc(var(--cell-border-dash-interval, 4px) * 2),
    transparent calc(var(--cell-border-dash-interval, 4px) * 2),
    transparent calc(var(--cell-border-dash-interval, 4px) * 2.5),
    var(--cell-border-color) calc(var(--cell-border-dash-interval, 4px) * 2.5),
    var(--cell-border-color) calc(var(--cell-border-dash-interval, 4px) * 2.5 + 1px),
    transparent calc(var(--cell-border-dash-interval, 4px) * 2.5 + 1px),
    transparent calc(var(--cell-border-dash-interval, 4px) * 3)
  ) 1;
}

/* 双点划线 - 使用 border-image 模拟，注意不支持圆角 */
.calendar-cell.border-style-dash-dot-dot {
  border-style: solid;
  border-image: repeating-linear-gradient(
    90deg,
    var(--cell-border-color) 0px,
    var(--cell-border-color) calc(var(--cell-border-dash-interval, 4px) * 2),
    transparent calc(var(--cell-border-dash-interval, 4px) * 2),
    transparent calc(var(--cell-border-dash-interval, 4px) * 2.5),
    var(--cell-border-color) calc(var(--cell-border-dash-interval, 4px) * 2.5),
    var(--cell-border-color) calc(var(--cell-border-dash-interval, 4px) * 2.5 + 1px),
    transparent calc(var(--cell-border-dash-interval, 4px) * 2.5 + 1px),
    transparent calc(var(--cell-border-dash-interval, 4px) * 3),
    var(--cell-border-color) calc(var(--cell-border-dash-interval, 4px) * 3),
    var(--cell-border-color) calc(var(--cell-border-dash-interval, 4px) * 3 + 1px),
    transparent calc(var(--cell-border-dash-interval, 4px) * 3 + 1px),
    transparent calc(var(--cell-border-dash-interval, 4px) * 3.5)
  ) 1;
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

/* 条目级别悬停效果 */
.schedule-item:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.dark .schedule-item:hover {
  background-color: rgba(255, 255, 255, 0.08);
}

/* 单元格悬停显示编辑按钮 */
.calendar-cell:hover .edit-btn {
  opacity: 1;
}

/* 编辑状态始终显示保存按钮 */
.calendar-cell.editing .edit-btn {
  opacity: 1;
}

/* 拖拽悬停效果 */
.calendar-cell.drag-over {
  outline: 2px dashed var(--primary);
  outline-offset: -2px;
  background-color: color-mix(in srgb, var(--primary) 10%, transparent);
}

/* 拖拽中的条目样式 */
.schedule-item:active {
  cursor: grabbing;
}

.schedule-item.dragging {
  opacity: 0.5;
}

/* 列表标记悬停效果 */
.marker-btn:hover .marker-dot {
  opacity: 0;
}

.marker-btn:hover .marker-delete {
  opacity: 1;
}
</style>
