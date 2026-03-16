<script setup lang="ts">
import { ref, onMounted, nextTick, computed, onUnmounted, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { X, Trash2, LayoutList, Plus } from 'lucide-vue-next';
import {
  loadMainTasks,
  searchMainTasks,
  saveMainTask,
  updateMainTaskContent,
  toggleMainTaskStatus,
  deleteMainTask,
  type MainTask
} from '../composables/db';
import { initDatabase } from '../composables/db/connection';
import { hexToRgba, adjustBrightness } from '../utils/color';
import type { AppSettings } from '../types';
import { defaultLightSettings, defaultDarkSettings } from '../types';

const settings = ref<AppSettings>({ ...defaultLightSettings });
const tasks = ref<MainTask[]>([]);
const newTaskContent = ref('');
const searchKeyword = ref('');

// 搜索框焦点状态
const isSearchFocused = ref(false);

// DOM Refs
const searchInputRef = ref<HTMLInputElement | null>(null);

// 动态主题样式
const themeStyle = computed(() => {
  const s = settings.value;
  const bgOpacity = s.bg_opacity / 100;
  const cellOpacity = s.cell_opacity / 100;
  return {
    '--theme-bg': hexToRgba(s.bg_color, bgOpacity),
    '--theme-cell': hexToRgba(s.cell_color, cellOpacity),
    '--theme-text': s.text_color,
    '--theme-text-secondary': adjustBrightness(s.text_color, 30),
    '--theme-text-muted': adjustBrightness(s.text_color, 50),
    '--theme-primary': s.primary_color,
    '--theme-primary-alpha': hexToRgba(s.primary_color, 0.2), // 用于选中文本的柔和高亮
    '--theme-border': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.05)'),
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
    const defaults = parsed.theme_mode === 'dark' ? defaultDarkSettings : defaultLightSettings;
    settings.value = { ...defaults, ...parsed };
  }
  applyTheme();
}

function applyTheme() {
  const s = settings.value;
  const root = document.documentElement;

  root.style.setProperty('--primary', s.primary_color);
  root.style.setProperty('--text-primary', s.text_color);
  root.style.setProperty('--text-muted', adjustBrightness(s.text_color, 60));
  root.setAttribute('data-theme', s.theme_mode);
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

async function handleAddTask() {
  const content = newTaskContent.value.trim();
  if (!content) return;

  try {
    await saveMainTask(content);
    newTaskContent.value = '';
    await loadTasks();
  } catch (error) {
    console.error('Failed to add task:', error);
  }
}

async function handleToggleDone(task: MainTask) {
  if (!task.id) return;
  try {
    await toggleMainTaskStatus(task.id, !task.is_done);
    await loadTasks();
  } catch (error) {
    console.error('Failed to toggle task:', error);
  }
}

async function handleDeleteTask(taskId: number) {
  try {
    await deleteMainTask(taskId);
    await loadTasks();
  } catch (error) {
    console.error('Failed to delete task:', error);
  }
}

async function handleUpdateTask(task: MainTask, newContent: string) {
  if (!task.id) return;
  const trimmed = newContent.trim();
  if (!trimmed) {
    // 如果内容为空，删除任务
    await handleDeleteTask(task.id);
    return;
  }
  if (trimmed === task.content) return;
  try {
    await updateMainTaskContent(task.id, trimmed);
    await loadTasks();
  } catch (error) {
    console.error('Failed to update task:', error);
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

onMounted(async () => {
  loadSettings();
  await initDatabase();
  await loadTasks();
  window.addEventListener('storage', handleSettingsUpdate);

  await nextTick();
  requestAnimationFrame(async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
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
            class="text-[14px] font-medium leading-relaxed transition-opacity cursor-text"
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
            class="absolute inset-0 w-full h-full bg-black/5 dark:bg-white/5 rounded-md px-2 outline-none text-[13px] leading-relaxed text-center selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
            :style="{ color: 'var(--theme-text)' }"
            @input="loadTasks"
            @focus="isSearchFocused = true"
            @blur="isSearchFocused = false"
          />
        </div>

        <button @click="handleAddTask"
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
        <TransitionGroup name="list" tag="div" class="space-y-2">
          <div key="inline-add-task"
            class="note-item group flex items-start gap-2.5 px-3 py-2.5 rounded-lg transition-all duration-300 cursor-text"
            :style="{
              backgroundColor: 'var(--theme-cell)',
              border: '1px solid var(--theme-border)',
            }">
            <div class="note-dot shrink-0 w-1.5 h-1.5 rounded-full mt-[7px] transition-all duration-200"
              :style="{ backgroundColor: 'var(--theme-text-muted)', opacity: 0.4 }"></div>
            <input v-model="newTaskContent" type="text" placeholder="..."
              class="inline-add-input flex-1 w-full bg-transparent p-0 text-[13px] leading-relaxed outline-none transition-all placeholder:transition-opacity focus:placeholder:opacity-40 selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
              :style="{ color: 'var(--theme-text)' }" @keydown.enter="handleAddTask"
              @keydown.escape="newTaskContent = ''" />
          </div>

          <div v-for="task in tasks" :key="task.id"
            class="note-item w-full flex items-start gap-2.5 px-3 py-2.5 rounded-lg transition-all duration-300 hover:bg-black/5 dark:hover:bg-white/5 hover:shadow-sm"
            :style="{
              backgroundColor: task.is_done ? 'transparent' : 'var(--theme-cell)',
              border: '1px solid',
              borderColor: task.is_done ? 'transparent' : 'var(--theme-border)',
            }"
            :class="task.is_done ? 'opacity-50' : ''"
            @contextmenu.prevent="handleToggleDone(task)">
            <div class="note-dot shrink-0 w-1.5 h-1.5 rounded-full mt-[7px] transition-all duration-200"
              :style="{ backgroundColor: 'var(--theme-text-muted)', opacity: 0.4 }"></div>

            <input type="text"
              :value="task.content"
              class="flex-1 min-w-0 w-full bg-transparent outline-none p-0 text-[13px] font-medium leading-relaxed selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
              :style="{ color: 'var(--theme-text)' }"
              :class="{ 'line-through decoration-gray-400/50': task.is_done }"
              @blur="(($event.target as HTMLInputElement).value.trim() !== task.content) && handleUpdateTask(task, ($event.target as HTMLInputElement).value)"
              @keydown.enter="($event.target as HTMLInputElement).blur()"
              @keydown.escape="($event.target as HTMLInputElement).value = task.content; ($event.target as HTMLInputElement).blur()" />

            <button @click.stop="handleDeleteTask(task.id!)"
              class="note-delete shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 hover:bg-red-50 dark:hover:bg-red-900/30 mt-0.5"
              :style="{ color: 'var(--theme-text-muted)' }">
              <Trash2 class="w-3.5 h-3.5 hover:text-red-500 dark:hover:text-red-400 transition-colors" />
            </button>
          </div>
        </TransitionGroup>

        <div v-if="tasks.length === 0" class="flex flex-col items-center justify-center py-20 pointer-events-none transition-opacity">
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

.inline-add-input::placeholder {
  color: var(--theme-text-muted);
}

/* 列表动画 */
.list-move,
.list-enter-active,
.list-leave-active {
  transition: all 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateY(4px) scale(0.99);
}

.list-leave-active {
  position: absolute;
  width: calc(100% - 24px);
}

/* 交互反馈：悬浮变色、显示删除按钮 */
.note-item:hover .note-dot {
  background-color: var(--theme-primary) !important;
  opacity: 1 !important;
  transform: scale(1.1);
}

.note-item:hover .note-delete {
  opacity: 1;
}
</style>