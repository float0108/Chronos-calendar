<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { emit } from '@tauri-apps/api/event';
import { X, Search, XCircle } from 'lucide-vue-next';
import ListItem from '../components/ListItem.vue';
import { searchSchedules } from '../composables/db';
import { initDatabase } from '../composables/db/connection';
import { hexToRgba, adjustBrightness } from '../utils/color';
import type { AppSettings, Schedule } from '../types';
import { defaultLightSettings, defaultDarkSettings } from '../types';

const settings = ref<AppSettings>({ ...defaultLightSettings });
const searchKeyword = ref('');
const searchResults = ref<Schedule[]>([]);
const isLoading = ref(false);

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
    '--theme-text-muted': s.muted_text_color,
    '--theme-primary': s.primary_color,
    '--theme-primary-alpha': hexToRgba(s.primary_color, 0.2),
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
  root.style.setProperty('--text-muted', s.muted_text_color);
  root.setAttribute('data-theme', s.theme_mode);
}

function handleSettingsUpdate() {
  loadSettings();
}

// 防抖搜索
let searchTimeout: ReturnType<typeof setTimeout> | null = null;

async function performSearch() {
  if (!searchKeyword.value.trim()) {
    searchResults.value = [];
    return;
  }

  isLoading.value = true;
  try {
    searchResults.value = await searchSchedules(searchKeyword.value);
  } catch (error) {
    console.error('Search failed:', error);
    searchResults.value = [];
  } finally {
    isLoading.value = false;
  }
}

watch(searchKeyword, () => {
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
  searchTimeout = setTimeout(() => {
    performSearch();
  }, 200);
});

// 点击搜索结果，导航到主窗口对应日期
async function handleResultClick(schedule: Schedule) {
  // 判断使用哪个日期和视图模式
  // 如果已完成且有 done_date，切换到 done 视图并定位到 done_date
  // 如果未完成或有 create_date，切换到 todo 视图并定位到 create_date
  const isDone = schedule.is_done;
  const targetDate = isDone && schedule.done_date ? schedule.done_date : schedule.create_date;
  const viewMode = isDone && schedule.done_date ? 'done' : 'todo';

  // 发送事件到主窗口
  await emit('navigate-to-date', {
    date: targetDate,
    viewMode: viewMode,
  });

  // 不关闭搜索窗口，允许用户继续搜索
}

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
  window.addEventListener('storage', handleSettingsUpdate);

  await nextTick();
  requestAnimationFrame(async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
    // 自动聚焦搜索框
    searchInputRef.value?.focus();
  });
});

onUnmounted(() => {
  window.removeEventListener('storage', handleSettingsUpdate);
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
});
</script>

<template>
  <div class="search-overlay fixed inset-0 z-[9999] flex w-full h-full" :style="themeStyle">
    <div class="search-panel relative flex flex-col overflow-hidden w-full h-full rounded-lg transition-colors shadow-lg"
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
          title="拖拽">
          <Search class="w-4 h-4" />
        </button>

        <div class="flex-1 min-w-0 relative">
          <input
            ref="searchInputRef"
            v-model="searchKeyword"
            type="text"
            placeholder="搜索日程..."
            class="w-full h-7 bg-black/5 dark:bg-white/5 rounded-md pl-3 pr-8 outline-none text-sm leading-relaxed selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
            :style="{ color: 'var(--theme-text)' }"
          />
          <button
            v-if="searchKeyword"
            @click="searchKeyword = ''"
            class="absolute right-1.5 top-1/2 -translate-y-1/2 w-5 h-5 flex items-center justify-center rounded transition-colors hover:bg-black/10 dark:hover:bg-white/10"
            :style="{ color: 'var(--theme-text-muted)' }"
          >
            <XCircle class="w-3.5 h-3.5" />
          </button>
        </div>

        <button @click="handleClose"
          class="close-btn shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
          :style="{ color: 'var(--theme-text)' }">
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto custom-scrollbar px-3 pt-2 pb-3">
        <!-- 加载状态 -->
        <div v-if="isLoading" class="flex items-center justify-center py-8">
          <div class="animate-spin w-5 h-5 border-2 border-[var(--theme-primary)] border-t-transparent rounded-full"></div>
        </div>

        <!-- 搜索结果列表 -->
        <div v-else-if="searchResults.length > 0" class="space-y-2">
          <ListItem
            v-for="schedule in searchResults"
            :key="schedule.id"
            :title="schedule.content"
            :preview="schedule.description"
            :date="schedule.create_date"
            :is-done="schedule.is_done"
            :editable="false"
            @click="handleResultClick(schedule)"
          />
        </div>

        <!-- 空状态 -->
        <div v-else-if="searchKeyword.trim()" class="flex flex-col items-center justify-center py-12 pointer-events-none">
          <div class="p-4 rounded-full" :style="{ backgroundColor: 'var(--theme-cell)' }">
            <Search class="w-6 h-6 opacity-30" :style="{ color: 'var(--theme-text)' }" />
          </div>
          <div class="mt-3 text-sm opacity-50" :style="{ color: 'var(--theme-text-muted)' }">
            未找到相关日程
          </div>
        </div>

        <!-- 初始状态 -->
        <div v-else class="flex flex-col items-center justify-center py-12 pointer-events-none">
          <div class="p-4 rounded-full" :style="{ backgroundColor: 'var(--theme-cell)' }">
            <Search class="w-6 h-6 opacity-30" :style="{ color: 'var(--theme-text)' }" />
          </div>
          <div class="mt-3 text-sm opacity-50" :style="{ color: 'var(--theme-text-muted)' }">
            输入关键词搜索日程
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
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

input {
  -webkit-appearance: none;
  appearance: none;
}
</style>
