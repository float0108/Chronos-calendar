<script setup lang="ts">
import { computed } from 'vue';
import { ChevronLeft, ChevronRight, MoreVertical, Lock, Unlock, Undo2, ListTodo, CheckCircle2, Kanban, StickyNote, Search } from 'lucide-vue-next';
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
  isSearchVisible?: boolean;
  hideWeekends: boolean;
}>();

const emit = defineEmits<{
  (e: 'prevMonth'): void;
  (e: 'nextMonth'): void;
  (e: 'goToday'): void;
  (e: 'toggleMiniCalendar'): void;
  (e: 'toggleMenu'): void;
  (e: 'selectDate', date: dayjs.Dayjs): void;
  (e: 'settings'): void;
  (e: 'hide'): void;
  (e: 'quit'): void;
  (e: 'toggleLock'): void;
  (e: 'exportBackup'): void;
  (e: 'exportZip'): void;
  (e: 'importBackup'): void;
  (e: 'sync'): void;
  (e: 'undo'): void;
  (e: 'redo'): void;
  (e: 'switchViewMode', mode: ViewMode): void;
  (e: 'toggleBoard'): void;
  (e: 'toggleNote'): void;
  (e: 'toggleSearch'): void;
  (e: 'toggleWeekends'): void;
  (e: 'openBatchTask'): void;
}>();

function handlePrev() { emit('prevMonth'); }
function handleNext() { emit('nextMonth'); }
function handleToday() { emit('goToday'); }
function handleToggleMini() { emit('toggleMiniCalendar'); }
function handleToggleMenu() { emit('toggleMenu'); }
function handleSelectDate(date: dayjs.Dayjs) { emit('selectDate', date); }
function handleMiniCalendarClose() { emit('toggleMiniCalendar'); }
function handleSettings() { emit('settings'); }
function handleHide() { emit('hide'); }
function handleQuit() { emit('quit'); }
function handleToggleLock() { emit('toggleLock'); }
function handleExportBackup() { emit('exportBackup'); }
function handleExportZip() { emit('exportZip'); }
function handleImportBackup() { emit('importBackup'); }
function handleSync() { emit('sync'); }
function handleUndo() { emit('undo'); }
function handleRedo() { emit('redo'); }
function handleSwitchViewMode(mode: ViewMode) { emit('switchViewMode', mode); }
function handleToggleBoard() { emit('toggleBoard'); }
function handleToggleNote() { emit('toggleNote'); }
function handleToggleSearch() { emit('toggleSearch'); }
function handleToggleWeekends() { emit('toggleWeekends'); }
function handleOpenBatchTask() { emit('openBatchTask'); }
function handleStartDrag(event: MouseEvent) { startWindowDrag(event); }

// 计算边框样式
const headerBorderStyle = computed(() => {
  const style = getComputedStyle(document.documentElement);
  const borderStyle = style.getPropertyValue('--cell-border-style').trim() || 'solid';

  // dashed 和 dotted 使用原生支持，dash-dot/dash-dot-dot 回退到 dashed
  let validBorderStyle: 'solid' | 'dashed' | 'dotted' = 'solid';
  if (borderStyle === 'dashed' || borderStyle === 'dotted') {
    validBorderStyle = borderStyle;
  } else if (borderStyle === 'dash-dot' || borderStyle === 'dash-dot-dot') {
    validBorderStyle = 'dashed';
  }

  return {
    backgroundColor: 'var(--header-bg)',
    borderBottomWidth: 'var(--cell-border-width)',
    borderBottomStyle: validBorderStyle,
    borderBottomColor: 'var(--header-border-color)',
  };
});
</script>

<template>
  <div class="calendar-header flex items-center justify-between px-4 py-3 select-none relative" :style="headerBorderStyle" @mousedown="handleStartDrag">
    <div class="absolute inset-0 z-0"></div>

    <div class="flex items-center gap-2 relative z-10 flex-1 justify-start">
      <div class="flex items-center p-0.5 rounded-lg bg-[var(--hover-bg)] dark:bg-gray-800/50">
        <button @mousedown.stop @click="handleToggleBoard"
          class="p-1.5 rounded-md transition-colors flex items-center justify-center"
          :class="isBoardVisible ? 'bg-[var(--primary-light)] text-[var(--primary)] shadow-sm' : 'hover:bg-[var(--hover-bg)]'"
          :style="isBoardVisible ? {} : { color: 'var(--text-muted)' }"
          :title="isBoardVisible ? '隐藏看板' : '显示看板'">
          <Kanban class="w-4 h-4" />
        </button>
        <button @mousedown.stop @click="handleToggleNote"
          class="p-1.5 rounded-md transition-colors flex items-center justify-center"
          :class="isNoteVisible ? 'bg-[var(--primary-light)] text-[var(--primary)] shadow-sm' : 'hover:bg-[var(--hover-bg)]'"
          :style="isNoteVisible ? {} : { color: 'var(--text-muted)' }"
          :title="isNoteVisible ? '隐藏备忘录' : '显示备忘录'">
          <StickyNote class="w-4 h-4" />
        </button>
        <button @mousedown.stop @click="handleToggleSearch"
          class="p-1.5 rounded-md transition-colors flex items-center justify-center"
          :class="isSearchVisible ? 'bg-[var(--primary-light)] text-[var(--primary)] shadow-sm' : 'hover:bg-[var(--hover-bg)]'"
          :style="isSearchVisible ? {} : { color: 'var(--text-muted)' }"
          :title="isSearchVisible ? '隐藏搜索' : '显示搜索'">
          <Search class="w-4 h-4" />
        </button>
      </div>

      <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div>

      <div class="flex items-center p-0.5 rounded-lg bg-[var(--hover-bg)] dark:bg-gray-800/50">
        <button @mousedown.stop @click="handleSwitchViewMode('todo')"
          class="p-1.5 rounded-md transition-colors flex items-center justify-center"
          :class="viewMode === 'todo' ? 'bg-[var(--primary-light)] text-[var(--primary)] shadow-sm' : 'hover:bg-[var(--hover-bg)]'"
          :style="viewMode === 'todo' ? {} : { color: 'var(--text-muted)' }"
          title="待办视图">
          <ListTodo class="w-4 h-4" />
        </button>
        <button @mousedown.stop @click="handleSwitchViewMode('done')"
          class="p-1.5 rounded-md transition-colors flex items-center justify-center"
          :class="viewMode === 'done' ? 'bg-[var(--primary-light)] text-[var(--primary)] shadow-sm' : 'hover:bg-[var(--hover-bg)]'"
          :style="viewMode === 'done' ? {} : { color: 'var(--text-muted)' }"
          title="完成视图">
          <CheckCircle2 class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div class="flex items-center gap-1.5 relative z-10 flex-1 justify-center">
      <div class="flex items-center gap-1.5 relative z-10 flex-1 justify-center shrink-0">
        <button @mousedown.stop @click="handlePrev"
          class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors shrink-0"
          :style="{ color: 'var(--text-muted)' }"
          title="上个月">
          <ChevronLeft class="w-4 h-4" />
        </button>

        <div class="relative shrink-0">
          <button @mousedown.stop @click="handleToggleMini"
            class="text-sm font-semibold min-w-[110px] text-center px-3 py-1 rounded-lg hover:bg-[var(--hover-bg)] transition-colors whitespace-nowrap"
            :style="{ color: 'var(--text-primary)' }">
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
          class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors shrink-0"
          :style="{ color: 'var(--text-muted)' }"
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
      <div class="flex items-center gap-0.5">
        <button @mousedown.stop @click="handleUndo"
          class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
          :class="{ 'opacity-30 pointer-events-none': !canUndo }"
          :style="{ color: 'var(--text-muted)' }"
          :disabled="!canUndo" title="撤销 (Ctrl+Z)">
          <Undo2 class="w-4 h-4" />
        </button>
        <button @mousedown.stop @click="handleRedo"
          class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
          :class="{ 'opacity-30 pointer-events-none': !canRedo }"
          :style="{ color: 'var(--text-muted)' }"
          :disabled="!canRedo" title="重做 (Ctrl+Y)">
          <Redo class="w-4 h-4" />
        </button>
      </div>

      <div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div>

      <button @mousedown.stop @click="handleToggleLock" class="p-1.5 rounded-lg transition-colors"
        :class="isLocked ? 'text-red-500 bg-red-50 dark:bg-red-900/20' : 'hover:bg-[var(--hover-bg)]'"
        :style="isLocked ? {} : { color: 'var(--text-muted)' }"
        :title="isLocked ? '已锁定 (点击解锁)' : '未锁定 (点击锁定)'">
        <Lock v-if="isLocked" class="w-4 h-4" />
        <Unlock v-else class="w-4 h-4" />
      </button>

      <div class="relative">
        <button @mousedown.stop @click="handleToggleMenu"
          class="p-1.5 rounded-lg hover:bg-[var(--hover-bg)] transition-colors"
          :class="{ 'opacity-50 pointer-events-none': isLocked }"
          :style="{ color: 'var(--text-muted)' }">
          <MoreVertical class="w-4 h-4" />
        </button>

        <DropdownMenu :visible="showMenu" :hide-weekends="hideWeekends" :is-locked="isLocked" @settings="handleSettings" @hide="handleHide" @quit="handleQuit" @exportBackup="handleExportBackup"
          @exportZip="handleExportZip" @importBackup="handleImportBackup" @sync="handleSync" @toggleWeekends="handleToggleWeekends" @openBatchTask="handleOpenBatchTask" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.calendar-header {
  cursor: default;
}
</style>