<script setup lang="ts">
import { ref, onMounted, nextTick, computed, onUnmounted, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { listen, emit } from '@tauri-apps/api/event';
import { X, LayoutList, Plus } from 'lucide-vue-next';
import ListItem from '../components/ListItem.vue';
import {
  loadMainTasks,
  searchMainTasks,
  saveMainTask,
  updateMainTaskContent,
  updateMainTaskCreateDate,
  toggleMainTaskStatus,
  deleteMainTask,
  type MainTask
} from '../composables/db';
import { hexToRgba, adjustBrightness } from '../utils/color';
import type { AppSettings } from '../types';
import { defaultLightSettings, defaultDarkSettings } from '../types';

const settings = ref<AppSettings>({ ...defaultLightSettings });
const tasks = ref<MainTask[]>([]);
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

async function loadTasks() {
  if (searchKeyword.value.trim()) {
    tasks.value = await searchMainTasks(searchKeyword.value);
  } else {
    tasks.value = await loadMainTasks();
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

async function handleAddTask(content: string) {
  const trimmed = content.trim();
  if (!trimmed) {
    isAdding.value = false;
    return;
  }

  try {
    // 保存新任务并获取返回的ID
    const newTaskId = await saveMainTask(trimmed);
    // 确保数据已写入数据库后再刷新列表
    await loadTasks();
    await notifyRefresh();
    // 如果有返回ID，自动打开新创建的任务窗口
    if (newTaskId) {
      const newTask = tasks.value.find(t => t.id === newTaskId);
      if (newTask) {
        await handleOpenTaskWindow(newTask);
      }
    }
  } catch (error) {
    console.error('Failed to add task:', error);
  }
  isAdding.value = false;
}

function handleCancelAdd() {
  isAdding.value = false;
}

async function handleToggleDone(task: MainTask) {
  if (!task.id) return;
  try {
    await toggleMainTaskStatus(task.id, !task.is_done);
    await loadTasks();
    await notifyRefresh();
  } catch (error) {
    console.error('Failed to toggle task:', error);
  }
}

async function handleDeleteTask(taskId: number) {
  try {
    await deleteMainTask(taskId);
    await loadTasks();
    await notifyRefresh();
  } catch (error) {
    console.error('Failed to delete task:', error);
  }
}

async function handleUpdateTask(task: MainTask, newContent: string) {
  if (!task.id) return;
  const trimmed = newContent.trim();
  if (!trimmed) {
    await handleDeleteTask(task.id);
    return;
  }
  if (trimmed === task.content) return;
  try {
    await updateMainTaskContent(task.id, trimmed);
    await loadTasks();
    await notifyRefresh();
  } catch (error) {
    console.error('Failed to update task:', error);
  }
}

async function handleUpdateTaskDate(task: MainTask, newDate: string) {
  if (!task.id) return;
  try {
    await updateMainTaskCreateDate(task.id, newDate);
    await loadTasks();
    await notifyRefresh();
  } catch (error) {
    console.error('Failed to update task date:', error);
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

async function handleOpenTaskWindow(task: MainTask) {
  if (!task.id) return;
  try {
    await invoke('open_task_window', { taskId: task.id });
  } catch (error) {
    console.error('Failed to open task window:', error);
  }
}

onMounted(async () => {
  loadSettings();
  await loadTasks();
  window.addEventListener('storage', handleSettingsUpdate);

  // 监听来自其他窗口的数据变更事件
  const unlisten = await listen('schedule-changed', async () => {
    await loadTasks();
  });

  await nextTick();
  requestAnimationFrame(async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
  });

  onUnmounted(() => {
    unlisten();
  });
});

onUnmounted(() => {
  window.removeEventListener('storage', handleSettingsUpdate);
});
</script>

<template>
  <div class="board-overlay fixed inset-0 z-[9999] flex w-full h-full" :style="themeStyle">
    <div class="board-panel relative flex flex-col overflow-hidden w-full h-full rounded-lg transition-colors shadow-lg"
      :style="{
        backgroundColor: 'var(--theme-bg)',
        border: '1px solid var(--theme-border)',
        backdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
        WebkitBackdropFilter: settings.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
      }">

      <div class="title-bar flex items-center gap-2 px-3 py-2.5 shrink-0 select-none group"
        data-tauri-drag-region>
        <button @mousedown="handleIconDrag"
          class="shrink-0 w-6 h-6 flex items-center justify-center cursor-grab active:cursor-grabbing hover:opacity-80 transition-opacity"
          :style="{ color: 'var(--theme-text)' }"
          title="Drag">
          <LayoutList class="w-4 h-4" />
        </button>

        <div class="flex-1 min-w-0 flex justify-center items-center relative h-6"
          @mousedown="(e) => e.target === e.currentTarget && handleIconDrag()">
          <span v-show="!isSearchFocused"
            class="text-base font-medium leading-relaxed transition-opacity"
            :style="{ color: 'var(--theme-text)' }"
            @click="isSearchFocused = true">
            Board
          </span>
          <input
            ref="searchInputRef"
            v-show="isSearchFocused"
            v-model="searchKeyword"
            type="text"
            placeholder="..."
            class="absolute inset-0 w-full h-full bg-black/5 dark:bg-white/5 rounded-md px-2 outline-none text-sm leading-relaxed text-center selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
            :style="{ color: 'var(--theme-text)' }"
            @input="loadTasks"
            @focus="isSearchFocused = true"
            @blur="isSearchFocused = false"
          />
        </div>

        <button @click="handleStartAdding"
          class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
          :style="{ color: 'var(--theme-text)' }">
          <Plus class="w-4 h-4" />
        </button>

        <button @click="handleClose"
          class="close-btn shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
          :style="{ color: 'var(--theme-text)' }">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
        <div class="space-y-2">
          <ListItem
            v-if="isAdding"
            key="add-new-task"
            is-add-mode
            @add="handleAddTask"
            @cancel="handleCancelAdd"
            @click.stop
          />

          <ListItem
            v-for="task in tasks"
            :key="task.id"
            :title="task.content"
            :date="task.create_date"
            :is-done="task.is_done"
            center-calendar
            @update:title="(val) => handleUpdateTask(task, val)"
            @update:date="(val) => handleUpdateTaskDate(task, val)"
            @delete="handleDeleteTask(task.id!)"
            @toggle-done="handleToggleDone(task)"
            @click="handleOpenTaskWindow(task)"
          />
        </div>

        <div v-if="tasks.length === 0 && !isAdding" class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity">
          <div class="p-4 rounded-full" :style="{ backgroundColor: 'var(--theme-cell)' }">
            <LayoutList class="w-8 h-8 opacity-20" :style="{ color: 'var(--theme-text)' }" />
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
