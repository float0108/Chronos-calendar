<script setup lang="ts">
import { ChevronLeft, ChevronRight, Calendar, MoreVertical, Lock, Unlock, Undo2, Redo2, ListTodo, CheckCircle2 } from 'lucide-vue-next';
import dayjs from 'dayjs';
import MiniCalendar from './MiniCalendar.vue';
import DropdownMenu from './DropdownMenu.vue';
import { formatMonthYear } from '../utils/date';
import { startWindowDrag } from '../utils/window';
import type { ViewMode } from '../types';

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
}>();

function handleDrag(event: MouseEvent) {
  startWindowDrag(event);
}

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
</script>

<template>
  <div 
    class="calendar-header flex items-center justify-between px-4 py-3 border-b border-[var(--border-light)] select-none"
    @mousedown="handleDrag"
  >
    <div class="flex items-center gap-2">
      <Calendar class="w-5 h-5 text-[var(--primary)]" />
      <span class="font-semibold text-lg text-gray-900">Chronos</span>
    </div>
    
    <div class="flex items-center gap-1 no-drag">
      <button 
        @click="handlePrev"
        class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
        :class="{ 'opacity-50 pointer-events-none': isLocked }"
      >
        <ChevronLeft class="w-4 h-4 text-gray-700" />
      </button>
      
      <div class="relative">
        <button 
          @click="handleToggleMini"
          class="text-sm font-medium min-w-[100px] text-center text-gray-900 px-2 py-1 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
          :class="{ 'opacity-50 pointer-events-none': isLocked }"
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
        @click="handleNext"
        class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
        :class="{ 'opacity-50 pointer-events-none': isLocked }"
      >
        <ChevronRight class="w-4 h-4 text-gray-700" />
      </button>
      
      <button
        @click="handleToday"
        class="ml-2 px-2 py-1 text-xs rounded-lg bg-[var(--primary-light)] text-[var(--primary)] hover:bg-[var(--primary-light-hover)] transition-colors"
        :class="{ 'opacity-50 pointer-events-none': isLocked }"
      >
        今天
      </button>

      <div class="flex items-center gap-0.5 ml-2 rounded-lg p-0.5">
        <button
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
          @click="handleUndo"
          class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
          :class="{ 'opacity-30 pointer-events-none': !canUndo }"
          :disabled="!canUndo"
          title="撤销 (Ctrl+Z)"
        >
          <Undo2 class="w-4 h-4 text-gray-700" />
        </button>

        <button
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
        @click="handleToggleLock"
        class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors ml-1"
        :title="isLocked ? '解锁' : '锁定'"
      >
        <Lock v-if="isLocked" class="w-4 h-4 text-red-500" />
        <Unlock v-else class="w-4 h-4 text-gray-500" />
      </button>
      
      <div class="relative">
        <button 
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
