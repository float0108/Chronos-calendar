<script setup lang="ts">
import { ref, onMounted, nextTick, computed, onUnmounted, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen, emit } from '@tauri-apps/api/event';
import { Plus, CheckSquare } from 'lucide-vue-next';
import ListItem from '../components/ListItem.vue';
import WindowTitleBar from '../components/WindowTitleBar.vue';
import {
  loadTodoSchedules,
  searchSchedules,
  toggleScheduleStatus,
  updateScheduleContent,
  updateScheduleDate,
  deleteSchedule,
  type Schedule
} from '../api/database';
import { hexToRgba, adjustBrightness } from '../utils/color';
import type { AppSettings } from '../types';
import { defaultLightSettings, defaultDarkSettings } from '../types';
import dayjs from 'dayjs';

const settings = ref<AppSettings>({ ...defaultLightSettings });
const schedules = ref<Schedule[]>([]);
const searchKeyword = ref('');

// 实际主题（解析 system）
const effectiveTheme = computed(() => {
  if (settings.value.theme_mode === 'system') {
    return document.documentElement.getAttribute('data-theme') as 'light' | 'dark' || 'light';
  }
  return settings.value.theme_mode;
});

// 搜索框焦点状态
const isSearchFocused = ref(false);

// 新增模式
const isAdding = ref(false);

// DOM Refs
const searchInputRef = ref<HTMLInputElement | null>(null);

// 动态主题样式
const themeStyle = computed(() => {
  const s = settings.value;
  const bgOpacity = s.bg_opacity / 100;
  const cellOpacity = s.cell_opacity / 100;
  const theme = effectiveTheme.value;
  return {
    '--theme-bg': hexToRgba(s.bg_color, bgOpacity),
    '--theme-cell': hexToRgba(s.cell_color, cellOpacity),
    '--theme-text': s.text_color,
    '--theme-text-secondary': adjustBrightness(s.text_color, 30),
    '--theme-text-muted': s.muted_text_color,
    '--theme-primary': s.primary_color,
    '--theme-primary-alpha': hexToRgba(s.primary_color, 0.2),
    '--theme-border': s.cell_border_color || (theme === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.05)'),
    '--theme-font-family': s.font_family,
    '--theme-font-size': `${s.font_size}px`,
    'font-family': s.font_family,
    'font-size': `${s.font_size}px`,
  };
});

function loadSettings() {
  const saved = localStorage.getItem('chronos_settings');
  if (saved) {
    const parsed = JSON.parse(saved);
    const actualTheme = parsed.theme_mode === 'system'
      ? (document.documentElement.getAttribute('data-theme') as 'light' | 'dark' || 'light')
      : (parsed.theme_mode || 'light');
    const defaults = actualTheme === 'dark' ? defaultDarkSettings : defaultLightSettings;
    settings.value = { ...defaults, ...parsed };
  }
  applyTheme();
}

function applyTheme() {
  const s = settings.value;
  const root = document.documentElement;
  root.style.setProperty('--primary', s.primary_color);
  root.style.setProperty('--text-primary', s.text_color);
  root.style.setProperty('--text-muted', s.muted_text_color);
}

function handleSettingsUpdate() {
  loadSettings();
}

async function loadSchedulesData() {
  // 获取今天和未来一年的数据用于显示未完成日程
  const today = dayjs();
  const startDate = today.format('YYYY-MM-DD');
  const endDate = today.add(1, 'year').format('YYYY-MM-DD');

  if (searchKeyword.value.trim()) {
    // 搜索时过滤未完成的
    const allResults = await searchSchedules(searchKeyword.value);
    schedules.value = allResults.filter(s => !s.is_done);
  } else {
    schedules.value = await loadTodoSchedules(startDate, endDate);
  }
}

// 通知其他窗口刷新数据
async function notifyRefresh() {
  try {
    await emit('schedule-changed', {});
  } catch (error) {
    console.error('Failed to notify refresh:', error);
  }
}

function handleStartAdding() {
  isAdding.value = true;
}

async function handleAddSchedule(content: string) {
  const trimmed = content.trim();
  if (!trimmed) {
    isAdding.value = false;
    return;
  }

  try {
    const today = dayjs().format('YYYY-MM-DD');
    await import('../api/database').then(db => db.saveSchedule(today, trimmed, false));
    await loadSchedulesData();
    await notifyRefresh();
  } catch (error) {
    console.error('Failed to add schedule:', error);
  }
  isAdding.value = false;
}

function handleCancelAdd() {
  isAdding.value = false;
}

async function handleToggleDone(schedule: Schedule) {
  if (!schedule.id) return;
  try {
    await toggleScheduleStatus(schedule.id, !schedule.is_done);
    await loadSchedulesData();
    await notifyRefresh();
  } catch (error) {
    console.error('Failed to toggle schedule:', error);
  }
}

async function handleDeleteSchedule(scheduleId: number) {
  try {
    await deleteSchedule(scheduleId);
    await loadSchedulesData();
    await notifyRefresh();
  } catch (error) {
    console.error('Failed to delete schedule:', error);
  }
}

async function handleUpdateSchedule(schedule: Schedule, newContent: string) {
  if (!schedule.id) return;
  const trimmed = newContent.trim();
  if (!trimmed) {
    await handleDeleteSchedule(schedule.id);
    return;
  }
  if (trimmed === schedule.content) return;
  try {
    await updateScheduleContent(schedule.id, trimmed);
    await loadSchedulesData();
    await notifyRefresh();
  } catch (error) {
    console.error('Failed to update schedule:', error);
  }
}

async function handleUpdateScheduleDate(schedule: Schedule, newDate: string) {
  if (!schedule.id) return;
  try {
    await updateScheduleDate(schedule.id, 'create_date', newDate);
    await loadSchedulesData();
    await notifyRefresh();
  } catch (error) {
    console.error('Failed to update schedule date:', error);
  }
}

watch(isSearchFocused, (focused) => {
  if (focused) {
    nextTick(() => {
      searchInputRef.value?.focus();
    });
  }
});

async function handleClose() {
  const win = getCurrentWindow();
  await win.hide();
}

async function handleIconDrag() {
  try {
    const win = getCurrentWindow();
    await win.startDragging();
  } catch (error) {
    console.error('Drag failed:', error);
  }
}

let unlisten: (() => void) | undefined;

onMounted(async () => {
  loadSettings();

  // 监听来自其他窗口的数据变更事件
  unlisten = await listen('schedule-changed', async () => {
    await loadSchedulesData();
  });

  window.addEventListener('storage', handleSettingsUpdate);

  await loadSchedulesData();
  await nextTick();
  requestAnimationFrame(async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
  });
});

onUnmounted(() => {
  unlisten?.();
  window.removeEventListener('storage', handleSettingsUpdate);
});
</script>

<template>
  <div class="todo-overlay fixed inset-0 z-[9999] flex w-full h-full" :style="themeStyle">
    <div class="todo-panel relative flex flex-col overflow-hidden w-full h-full rounded-lg transition-colors shadow-lg"
      :style="{
        backgroundColor: 'var(--theme-bg)',
        border: '1px solid var(--theme-border)',
        backdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
        WebkitBackdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
      }">

      <WindowTitleBar
        :theme-style="themeStyle"
        @close="handleClose"
        @start-drag="handleIconDrag"
      >
        <template #left>
          <button @mousedown.stop="handleIconDrag"
            class="shrink-0 w-6 h-6 flex items-center justify-center cursor-grab active:cursor-grabbing hover:opacity-80 transition-opacity"
            :style="{ color: 'var(--theme-text)' }"
            title="Drag">
            <CheckSquare class="w-4 h-4" />
          </button>
        </template>

        <span v-show="!isSearchFocused"
          class="text-base font-medium leading-relaxed transition-opacity"
          :style="{ color: 'var(--theme-text)' }"
          @click="isSearchFocused = true">
          Todo
        </span>
        <input
          ref="searchInputRef"
          v-show="isSearchFocused"
          v-model="searchKeyword"
          type="text"
          placeholder="..."
          class="absolute inset-0 w-full h-full bg-black/5 dark:bg-white/5 rounded-md px-2 outline-none text-sm leading-relaxed text-center selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
          :style="{ color: 'var(--theme-text)' }"
          @input="loadSchedulesData"
          @focus="isSearchFocused = true"
          @blur="isSearchFocused = false"
          @mousedown.stop
        />

        <template #right>
          <button @click="handleStartAdding"
            class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
            :style="{ color: 'var(--theme-text)' }">
            <Plus class="w-4 h-4" />
          </button>
        </template>
      </WindowTitleBar>

      <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
        <div class="space-y-2">
          <ListItem
            v-if="isAdding"
            key="add-new-schedule"
            is-add-mode
            @add="handleAddSchedule"
            @cancel="handleCancelAdd"
            @click.stop
          />

          <ListItem
            v-for="schedule in schedules"
            :key="schedule.id"
            :title="schedule.content"
            :preview="schedule.description"
            :date="schedule.create_date"
            :is-done="schedule.is_done"
            center-calendar
            @update:title="(val) => handleUpdateSchedule(schedule, val)"
            @update:date="(val) => handleUpdateScheduleDate(schedule, val)"
            @delete="handleDeleteSchedule(schedule.id!)"
            @toggle-done="handleToggleDone(schedule)"
          />
        </div>

        <div v-if="schedules.length === 0 && !isAdding" class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity">
          <div class="p-4 rounded-full" :style="{ backgroundColor: 'var(--theme-cell)' }">
            <CheckSquare class="w-8 h-8 opacity-20" :style="{ color: 'var(--theme-text)' }" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 优雅的悬浮滚动条，默认隐藏，hover时显示 */
.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: transparent transparent;
  transition: scrollbar-color 0.3s ease;
}

.custom-scrollbar:hover {
  scrollbar-color: var(--theme-border) transparent;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: transparent;
  border-radius: 4px;
}

.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background-color: var(--theme-border);
}

::selection {
  background-color: var(--theme-primary-alpha);
  color: inherit;
}

input, textarea {
  -webkit-appearance: none;
  appearance: none;
}
</style>