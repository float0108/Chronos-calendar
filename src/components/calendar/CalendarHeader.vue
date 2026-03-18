<script setup lang="ts">
import { ChevronLeft, ChevronRight, MoreVertical, Lock, Unlock, Undo2, Redo2, ListTodo, CheckCircle2, CalendarPlus, Kanban, StickyNote } from 'lucide-vue-next';
import dayjs from 'dayjs';
import MiniCalendar from './MiniCalendar.vue';
import DropdownMenu from '../ui/DropdownMenu.vue';
import { formatMonthYear } from '../../utils/date';
import { startWindowDrag } from '../../utils/window';
import type { ViewMode } from '../../types';

defineProps<{
  currentDate: dayjs.Dayjs;
  showMiniCalendar: boolean;
  showMenu: boolean;
  isLocked: boolean;
  canUndo: boolean;
  canRedo: boolean;
  viewMode: ViewMode;
  isBoardVisible?: boolean;
  isNoteVisible?: boolean;
}>();

const emit = defineEmits<{
  (e: 'prevMonth'): void;
  (e: 'nextMonth'): void;
  (e: 'goToday'): void;
  (e: 'toggleMiniCalendar'): void;
  (e: 'toggleMenu'): void;
  (e: 'selectDate', date: dayjs.Dayjs): void;
  (e: 'settings'): void;
  (e: 'quit'): void;
  (e: 'toggleLock'): void;
  (e: 'export'): void;
  (e: 'import'): void;
  (e: 'undo'): void;
  (e: 'redo'): void;
  (e: 'switchViewMode', mode: ViewMode): void;
  (e: 'openBatchTask'): void;
  (e: 'toggleBoard'): void;
  (e: 'toggleNote'): void;
}>();

function handlePrev() { emit('prevMonth'); }
function handleNext() { emit('nextMonth'); }
function handleToday() { emit('goToday'); }
function handleToggleMini() { emit('toggleMiniCalendar'); }
function handleToggleMenu() { emit('toggleMenu'); }
function handleSelectDate(date: dayjs.Dayjs) { emit('selectDate', date); }
function handleMiniCalendarClose() { emit('toggleMiniCalendar'); }
function handleSettings() { emit('settings'); }
function handleQuit() { emit('quit'); }
function handleToggleLock() { emit('toggleLock'); }
function handleExport() { emit('export'); }
function handleImport() { emit('import'); }
function handleUndo() { emit('undo'); }
function handleRedo() { emit('redo'); }
function handleSwitchViewMode(mode: ViewMode) { emit('switchViewMode', mode); }
function handleOpenBatchTask() { emit('openBatchTask'); }
function handleToggleBoard() { emit('toggleBoard'); }
function handleToggleNote() { emit('toggleNote'); }
function handleStartDrag(event: MouseEvent) { startWindowDrag(event); }
</script>

<template>
  <div class="calendar-header flex items-center justify-between px-4 py-3 select-none relative" :style="{
    backgroundColor: 'var(--header-bg)',
    borderBottomWidth: 'var(--cell-border-width)',
    borderBottomStyle: 'solid',
    borderBottomColor: 'var(--header-border-color)'
  }" @mousedown="handleStartDrag">
    <div class="absolute inset-0 z-0"></div>

    <div class="flex items-center gap-2 relative z-10 flex-1 justify-start">
      <div class="flex items-center p-0.5 rounded-lg bg-[var(--hover-bg)] dark:bg-gray-800/50">
        <button @mousedown.stop @click="handleToggleBoard"
          class="p-1.5 rounded-md transition-colors flex items-center justify-center"
          :class="isBoardVisible ? 'bg-[var(--primary-light)] text-[var(--primary)] shadow-sm' : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200'"
          :title="isBoardVisible ? '隐藏看板' : '显示看板'">
          <Kanban class="w-4 h-4" />
        </button>
        <button @mousedown.stop @click="handleToggleNote"
          class="p-1.5 rounded-md transition-colors flex items-center justify-center"
          :class="isNoteVisible ? 'bg-[var(--primary-light)] text-[var(--primary)] shadow-sm' : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200'"
          :title="isNoteVisible ? '隐藏备忘录' : '显示备忘录'">
          <StickyNote class="w-4 h-4" />
        </button>
      </div>

      <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div>

      <div class="flex items-center p-0.5 rounded-lg bg-[var(--hover-bg)] dark:bg-gray-800/50">
        <button @mousedown.stop @click="handleSwitchViewMode('todo')"
          class="p-1.5 rounded-md transition-colors flex items-center justify-center"
          :class="viewMode === 'todo' ? 'bg-[var(--primary-light)] text-[var(--primary)] shadow-sm' : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200'"
          title="待办视图">
          <ListTodo class="w-4 h-4" />
        </button>
        <button @mousedown.stop @click="handleSwitchViewMode('done')"
          class="p-1.5 rounded-md transition-colors flex items-center justify-center"
          :class="viewMode === 'done' ? 'bg-[var(--primary-light)] text-[var(--primary)] shadow-sm' : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200'"
          title="完成视图">
          <CheckCircle2 class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div class="flex items-center gap-1.5 relative z-10 flex-1 justify-center">
      <div class="flex items-center gap-1.5 relative z-10 flex-1 justify-center shrink-0">
        <button @mousedown.stop @click="handlePrev"
          class="p-1.5 rounded-lg text-gray-600 hover:text-gray-900 hover:bg-[var(--hover-bg)] dark:text-gray-400 dark:hover:text-gray-200 transition-colors shrink-0"
          title="上个月">
          <ChevronLeft class="w-4 h-4" />
        </button>

        <div class="relative shrink-0">
          <button @mousedown.stop @click="handleToggleMini"
            class="text-sm font-semibold min-w-[110px] text-center text-gray-800 dark:text-gray-200 px-3 py-1 rounded-lg hover:bg-[var(--hover-bg)] transition-colors whitespace-nowrap">
            {{ formatMonthYear(currentDate) }}
          </button>

          <MiniCalendar
            :current-date="currentDate"
            :visible="showMiniCalendar"
            @select="handleSelectDate"
            @close="handleMiniCalendarClose"
          />
        </div>

        <button @mousedown.stop @click="handleNext"
          class="p-1.5 rounded-lg text-gray-600 hover:text-gray-900 hover:bg-[var(--hover-bg)] dark:text-gray-400 dark:hover:text-gray-200 transition-colors shrink-0"
          title="下个月">
          <ChevronRight class="w-4 h-4" />
        </button>

        <button @mousedown.stop @click="handleToday"
          class="ml-1 px-3 py-1 text-xs font-medium rounded-lg bg-[var(--primary-light)] text-[var(--primary)] hover:opacity-80 transition-opacity whitespace-nowrap shrink-0">
          今天
        </button>
      </div>
    </div>

    <div class="flex items-center gap-1.5 relative z-10 flex-1 justify-end">
      <button @mousedown.stop @click="handleOpenBatchTask"
        class="p-1.5 rounded-lg text-gray-600 hover:text-[var(--primary)] hover:bg-[var(--primary-light)] dark:text-gray-400 transition-colors"
        :class="{ 'opacity-50 pointer-events-none': isLocked }" :disabled="isLocked" title="批量添加任务">
        <CalendarPlus class="w-4 h-4" />
      </button>

      <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div>

      <div class="flex items-center gap-0.5">
        <button @mousedown.stop @click="handleUndo"
          class="p-1.5 rounded-lg text-gray-600 hover:text-gray-900 hover:bg-[var(--hover-bg)] dark:text-gray-400 dark:hover:text-gray-200 transition-colors"
          :class="{ 'opacity-30 pointer-events-none': !canUndo }" :disabled="!canUndo" title="撤销 (Ctrl+Z)">
          <Undo2 class="w-4 h-4" />
        </button>
        <button @mousedown.stop @click="handleRedo"
          class="p-1.5 rounded-lg text-gray-600 hover:text-gray-900 hover:bg-[var(--hover-bg)] dark:text-gray-400 dark:hover:text-gray-200 transition-colors"
          :class="{ 'opacity-30 pointer-events-none': !canRedo }" :disabled="!canRedo" title="重做 (Ctrl+Y)">
          <Redo2 class="w-4 h-4" />
        </button>
      </div>

      <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div>

      <button @mousedown.stop @click="handleToggleLock" class="p-1.5 rounded-lg transition-colors"
        :class="isLocked ? 'text-red-500 bg-red-50 dark:bg-red-900/20' : 'text-gray-600 hover:text-gray-900 hover:bg-[var(--hover-bg)] dark:text-gray-400 dark:hover:text-gray-200'"
        :title="isLocked ? '已锁定 (点击解锁)' : '未锁定 (点击锁定)'">
        <Lock v-if="isLocked" class="w-4 h-4" />
        <Unlock v-else class="w-4 h-4" />
      </button>

      <div class="relative">
        <button @mousedown.stop @click="handleToggleMenu"
          class="p-1.5 rounded-lg text-gray-600 hover:text-gray-900 hover:bg-[var(--hover-bg)] dark:text-gray-400 dark:hover:text-gray-200 transition-colors"
          :class="{ 'opacity-50 pointer-events-none': isLocked }">
          <MoreVertical class="w-4 h-4" />
        </button>

        <DropdownMenu :visible="showMenu" @settings="handleSettings" @quit="handleQuit" @export="handleExport"
          @import="handleImport" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.calendar-header {
  cursor: default;
}
</style>