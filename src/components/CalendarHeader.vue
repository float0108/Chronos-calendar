<script setup lang="ts">
import { ChevronLeft, ChevronRight, Calendar, MoreVertical, Lock, Unlock, Undo2, Redo2, ListTodo, CheckCircle2, CalendarPlus } from 'lucide-vue-next';
import dayjs from 'dayjs';
import MiniCalendar from './MiniCalendar.vue';
import DropdownMenu from './DropdownMenu.vue';
import { formatMonthYear } from '../utils/date';
import { startWindowDrag } from '../utils/window';
import type { ViewMode, AppSettings } from '../types';

defineProps<{
  currentDate: dayjs.Dayjs;
  showMiniCalendar: boolean;
  showMenu: boolean;
  isLocked: boolean;
  canUndo: boolean;
  canRedo: boolean;
  viewMode: ViewMode;
}>();

const emit = defineEmits<{
  (e: 'prevMonth'): void;
  (e: 'nextMonth'): void;
  (e: 'goToday'): void;
  (e: 'toggleMiniCalendar'): void;
  (e: 'toggleMenu'): void;
  (e: 'selectDate', day: number): void;
  (e: 'settings'): void;
  (e: 'quit'): void;
  (e: 'toggleLock'): void;
  (e: 'export'): void;
  (e: 'import'): void;
  (e: 'undo'): void;
  (e: 'redo'): void;
  (e: 'switchViewMode', mode: ViewMode): void;
  (e: 'openBatchTask'): void;
}>();

function handlePrev() {
  emit('prevMonth');
}

function handleNext() {
  emit('nextMonth');
}

function handleToday() {
  emit('goToday');
}

function handleToggleMini() {
  emit('toggleMiniCalendar');
}

function handleToggleMenu() {
  emit('toggleMenu');
}

function handleSelectDate(day: number) {
  emit('selectDate', day);
}

function handleSettings() {
  emit('settings');
}

function handleQuit() {
  emit('quit');
}

function handleToggleLock() {
  emit('toggleLock');
}

function handleExport() {
  emit('export');
}

function handleImport() {
  emit('import');
}

function handleUndo() {
  emit('undo');
}

function handleRedo() {
  emit('redo');
}

function handleSwitchViewMode(mode: ViewMode) {
  emit('switchViewMode', mode);
}

function handleOpenBatchTask() {
  emit('openBatchTask');
}

function handleStartDrag(event: MouseEvent) {
  startWindowDrag(event);
}
</script>

<template>
  <div
    class="calendar-header flex items-center justify-between px-4 py-3 select-none relative"
    :style="{
      backgroundColor: 'var(--header-bg)',
      borderBottomWidth: 'var(--cell-border-width)',
      borderBottomStyle: 'solid',
      borderBottomColor: 'var(--header-border-color)'
    }"
    @mousedown="handleStartDrag"
  >
    <!-- 拖动层 -->
    <div class="absolute inset-0 z-0"></div>

    <!-- 内容层（按钮等） -->
    <div class="flex items-center gap-2 relative z-10">
      <Calendar class="w-5 h-5 text-[var(--primary)]" />
      <span class="font-semibold text-lg text-gray-900">Chronos</span>
    </div>

    <div class="flex items-center gap-1 relative z-10">
      <button
        @mousedown.stop
        @click="handlePrev"
        class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
      >
        <ChevronLeft class="w-4 h-4 text-gray-700" />
      </button>

      <div class="relative">
        <button
          @mousedown.stop
          @click="handleToggleMini"
          class="text-sm font-medium min-w-[100px] text-center text-gray-900 px-2 py-1 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
        >
          {{ formatMonthYear(currentDate) }}
        </button>

        <MiniCalendar
          :current-date="currentDate"
          :visible="showMiniCalendar"
          @select="handleSelectDate"
          @prevMonth="handlePrev"
          @nextMonth="handleNext"
        />
      </div>

      <button
        @mousedown.stop
        @click="handleNext"
        class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
      >
        <ChevronRight class="w-4 h-4 text-gray-700" />
      </button>

      <button
        @mousedown.stop
        @click="handleToday"
        class="ml-2 px-2 py-1 text-xs rounded-lg bg-[var(--primary-light)] text-[var(--primary)] hover:bg-[var(--primary-light-hover)] transition-colors"
      >
        今天
      </button>

      <button
        @mousedown.stop
        @click="handleOpenBatchTask"
        class="ml-2 p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
        :class="{ 'opacity-50 pointer-events-none': isLocked }"
        :disabled="isLocked"
        title="批量添加任务"
      >
        <CalendarPlus class="w-4 h-4 text-gray-700" />
      </button>

      <div class="flex items-center gap-0.5 ml-2 rounded-lg p-0.5">
        <button
          @mousedown.stop
          @click="handleSwitchViewMode('todo')"
          class="p-1.5 rounded transition-colors"
          :class="viewMode === 'todo'
            ? 'text-[var(--primary)] bg-[var(--primary-light)]'
            : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-[var(--hover-bg)]'"
          :title="'待办视图'"
        >
          <ListTodo class="w-4 h-4" />
        </button>
        <button
          @mousedown.stop
          @click="handleSwitchViewMode('done')"
          class="p-1.5 rounded transition-colors"
          :class="viewMode === 'done'
            ? 'text-[var(--primary)] bg-[var(--primary-light)]'
            : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-[var(--hover-bg)]'"
          :title="'完成视图'"
        >
          <CheckCircle2 class="w-4 h-4" />
        </button>
      </div>

      <div class="flex items-center gap-1 ml-2">
        <button
          @mousedown.stop
          @click="handleUndo"
          class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
          :class="{ 'opacity-30 pointer-events-none': !canUndo }"
          :disabled="!canUndo"
          title="撤销 (Ctrl+Z)"
        >
          <Undo2 class="w-4 h-4 text-gray-700" />
        </button>

        <button
          @mousedown.stop
          @click="handleRedo"
          class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
          :class="{ 'opacity-30 pointer-events-none': !canRedo }"
          :disabled="!canRedo"
          title="重做 (Ctrl+Y)"
        >
          <Redo2 class="w-4 h-4 text-gray-700" />
        </button>
      </div>

      <button
        @mousedown.stop
        @click="handleToggleLock"
        class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors ml-1"
        :title="isLocked ? '解锁' : '锁定'"
      >
        <Lock v-if="isLocked" class="w-4 h-4 text-red-500" />
        <Unlock v-else class="w-4 h-4 text-gray-500" />
      </button>

      <div class="relative">
        <button
          @mousedown.stop
          @click="handleToggleMenu"
          class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
          :class="{ 'opacity-50 pointer-events-none': isLocked }"
        >
          <MoreVertical class="w-4 h-4 text-gray-700" />
        </button>

        <DropdownMenu
          :visible="showMenu"
          @settings="handleSettings"
          @quit="handleQuit"
          @export="handleExport"
          @import="handleImport"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.calendar-header {
  cursor: default;
}
</style>
