<script setup lang="ts">
import { ref, watch, onMounted, nextTick, computed } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { ask } from '@tauri-apps/plugin-dialog';
import type { AppSettings, ThemeMode } from '../types';
import { defaultLightSettings, defaultDarkSettings, extractCommonParts } from '../types/index';
import { hexToRgba, adjustBrightness } from '../utils/color';
import { useFonts } from '../composables/useFonts';
import CommonSettings from './CommonSettings.vue';
import ModeSettings from './ModeSettings.vue';

const { loadFonts, isFontsLoaded } = useFonts();

const activeMainTab = ref<'common' | 'mode'>('common');
const activeTab = ref<ThemeMode>('light');
const localSettings = ref<AppSettings>({ ...defaultLightSettings });
const originalSettings = ref<AppSettings | null>(null);
const hasChanges = ref(false);
const isRefreshingFonts = ref(false);
const fontsLoading = ref(!isFontsLoaded());

// 动态主题样式
const themeStyle = computed(() => {
  const s = localSettings.value;
  const bgOpacity = s.bg_opacity / 100;
  const cellOpacity = s.cell_opacity / 100;
  return {
    '--theme-bg': hexToRgba(s.bg_color, bgOpacity),
    '--theme-cell': hexToRgba(s.cell_color, cellOpacity),
    '--theme-cell-muted': hexToRgba(s.cell_color, cellOpacity * 0.6),
    '--theme-text': s.text_color,
    '--theme-text-secondary': adjustBrightness(s.text_color, 30),
    '--theme-text-muted': adjustBrightness(s.text_color, 50),
    '--theme-primary': s.primary_color,
    '--theme-primary-light': hexToRgba(s.primary_color, 0.1),
    '--theme-primary-hover': hexToRgba(s.primary_color, 0.2),
    '--theme-border': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.06)'),
    '--theme-font-family': s.font_family,
    '--theme-font-size': `${s.font_size}px`,
    'font-family': s.font_family,
    'font-size': `${s.font_size}px`,
  };
});

async function refreshFonts() {
  isRefreshingFonts.value = true;
  await loadFonts(true);
  isRefreshingFonts.value = false;
}

async function loadSettings() {
  try {
    const saved = localStorage.getItem('chronos_settings');
    if (saved) {
      const loadedSettings = JSON.parse(saved);
      const defaults = loadedSettings.theme_mode === 'dark' ? defaultDarkSettings : defaultLightSettings;
      localSettings.value = { ...defaults, ...loadedSettings };
      originalSettings.value = JSON.parse(JSON.stringify(localSettings.value));
      activeTab.value = loadedSettings.theme_mode;
      applySettings();
    }
  } catch (error) {
    console.error('Failed to load settings:', error);
  }
}

watch(localSettings, () => {
  if (originalSettings.value) {
    hasChanges.value = JSON.stringify(localSettings.value) !== JSON.stringify(originalSettings.value);
  }
}, { deep: true });

function applySettings() {
  const s = localSettings.value;
  const root = document.documentElement;
  const primary = s.primary_color;
  const textPrimary = s.text_color;
  const bgColor = s.bg_color;
  const bgOpacity = s.bg_opacity / 100;
  const cellOpacity = s.cell_opacity / 100;

  root.style.setProperty('--primary', primary);
  root.style.setProperty('--primary-light', hexToRgba(primary, 0.1));
  root.style.setProperty('--primary-light-hover', hexToRgba(primary, 0.2));

  root.style.setProperty('--text-primary', textPrimary);
  root.style.setProperty('--text-secondary', adjustBrightness(textPrimary, 20));
  root.style.setProperty('--text-muted', adjustBrightness(textPrimary, 60));

  root.style.setProperty('--glass-bg', hexToRgba(bgColor, bgOpacity));
  root.style.setProperty('--cell-bg', hexToRgba(s.cell_color, cellOpacity));
  root.style.setProperty('--cell-bg-muted', hexToRgba(s.cell_color, cellOpacity * 0.6));
  root.style.setProperty('--cell-bg-opacity', String(cellOpacity));

  root.style.setProperty('--hover-bg', hexToRgba('#000000', 0.05));
  root.style.setProperty('--schedule-bg', hexToRgba('#ffffff', cellOpacity + 0.2));
  root.style.setProperty('--schedule-hover', hexToRgba('#ffffff', cellOpacity + 0.4));

  root.style.setProperty('--border-light', hexToRgba('#000000', 0.1));
  root.style.setProperty('--cell-border-color', s.cell_border_color || '#d1d5db');
  root.style.setProperty('--backdrop-blur', s.enable_blur ? '16px' : '0px');
  root.style.setProperty('--cell-border-width', `${s.cell_border_width}px`);

  root.style.setProperty('--font-family-base', s.font_family);
  root.style.setProperty('--font-size-base', `${s.font_size}px`);
  root.style.setProperty('--font-weight-base', `${s.font_weight}`);
  root.style.setProperty('--cell-gap', `${s.cell_gap}px`);

  root.setAttribute('data-theme', s.theme_mode);
}

onMounted(async () => {
  await loadSettings();
  await nextTick();
  requestAnimationFrame(async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
  });
  loadFonts().finally(() => {
    fontsLoading.value = false;
  });
});

async function handleClose() {
  if (hasChanges.value) {
    const shouldClose = await ask('设置已修改但未保存，是否放弃修改？', {
      title: '确认关闭',
      kind: 'warning',
      okLabel: '放弃修改',
      cancelLabel: '取消',
    });
    if (!shouldClose) return;
  }
  getCurrentWindow().close();
}

async function handleSave() {
  const savedSettings = { ...localSettings.value, theme_mode: activeTab.value };
  try {
    localStorage.setItem('chronos_settings', JSON.stringify(savedSettings));
    localStorage.setItem(`chronos_settings_${activeTab.value}`, JSON.stringify(savedSettings));

    window.dispatchEvent(new StorageEvent('storage', {
      key: 'chronos_settings',
      newValue: JSON.stringify(savedSettings),
      url: window.location.href
    }));

    const commonParts = extractCommonParts(savedSettings);
    const otherMode = activeTab.value === 'light' ? 'dark' : 'light';
    const otherSaved = localStorage.getItem(`chronos_settings_${otherMode}`);
    if (otherSaved) {
      const otherSettings = JSON.parse(otherSaved);
      const updatedOther = { ...otherSettings, ...commonParts };
      localStorage.setItem(`chronos_settings_${otherMode}`, JSON.stringify(updatedOther));
    } else {
      const otherDefaults = otherMode === 'light' ? defaultLightSettings : defaultDarkSettings;
      localStorage.setItem(`chronos_settings_${otherMode}`, JSON.stringify({ ...otherDefaults, ...commonParts }));
    }

    hasChanges.value = false;
    getCurrentWindow().close();
  } catch (error) {
    console.error('Failed to save settings:', error);
  }
}

function handleSwitchMode(mode: ThemeMode) {
  activeTab.value = mode;
  const currentCommonParts = extractCommonParts(localSettings.value);

  const saved = localStorage.getItem(`chronos_settings_${mode}`);
  if (saved) {
    localSettings.value = { ...JSON.parse(saved), ...currentCommonParts, theme_mode: mode };
  } else {
    localSettings.value = mode === 'light'
      ? { ...defaultLightSettings, ...currentCommonParts, theme_mode: mode }
      : { ...defaultDarkSettings, ...currentCommonParts, theme_mode: mode };
  }
}

function handleReset() {
  const defaults = activeTab.value === 'light' ? defaultLightSettings : defaultDarkSettings;
  localSettings.value = { ...defaults };
}
</script>

<template>
  <div class="settings-overlay fixed inset-0 z-[9999] flex w-full h-full bg-transparent" :style="themeStyle">
    <div class="settings-panel relative backdrop-blur-2xl flex flex-col overflow-hidden w-full h-full rounded-none shadow-[0_32px_64px_-12px_rgba(0,0,0,0.15)]"
         :style="{
           backgroundColor: 'var(--theme-bg)',
           border: '1px solid var(--theme-border)'
         }">
      <!-- Header - 使用单元格颜色 -->
      <div class="flex items-center px-5 py-4 shrink-0 rounded-t-xl"
           :style="{ backgroundColor: 'var(--theme-cell)', borderBottom: '1px solid var(--theme-border)' }"
           data-tauri-drag-region>
        <div class="flex items-center gap-2">
          <span class="w-1.5 h-1.5 rounded-full animate-pulse" :style="{ backgroundColor: 'var(--theme-primary)' }"></span>
          <h1 class="text-[15px] font-bold tracking-tight" :style="{ color: 'var(--theme-text)' }">设置</h1>
        </div>
        <div class="flex-1" data-tauri-drag-region></div>
        <button @click="handleClose"
          class="w-9 h-9 flex items-center justify-center rounded-xl transition-all hover:bg-black/5"
          :style="{ color: 'var(--theme-text-muted)' }"
          title="关闭">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Tab Navigation - 使用单元格颜色 -->
      <div class="flex items-center px-5 py-2 gap-1 shrink-0"
           :style="{ backgroundColor: 'var(--theme-cell-muted)' }">
        <button @click="activeMainTab = 'common'"
          class="px-4 py-2 text-[13px] font-medium rounded-lg transition-all"
          :style="activeMainTab === 'common' ? {
            backgroundColor: 'var(--theme-cell)',
            color: 'var(--theme-text)',
            boxShadow: '0 1px 3px rgba(0,0,0,0.08)'
          } : { color: 'var(--theme-text-muted)' }">
          公用配置
        </button>
        <button @click="activeMainTab = 'mode'"
          class="px-4 py-2 text-[13px] font-medium rounded-lg transition-all"
          :style="activeMainTab === 'mode' ? {
            backgroundColor: 'var(--theme-cell)',
            color: 'var(--theme-text)',
            boxShadow: '0 1px 3px rgba(0,0,0,0.08)'
          } : { color: 'var(--theme-text-muted)' }">
          模式配置
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto">
        <CommonSettings
          v-show="activeMainTab === 'common'"
          :settings="localSettings"
          :fonts-loading="fontsLoading"
          v-model:is-refreshing-fonts="isRefreshingFonts"
          @update:settings="localSettings = $event"
          @refresh-fonts="refreshFonts"
        />

        <ModeSettings
          v-show="activeMainTab === 'mode'"
          :settings="localSettings"
          :active-tab="activeTab"
          @update:settings="localSettings = $event"
          @switch-mode="handleSwitchMode"
        />
      </div>

      <!-- Footer - 使用单元格颜色 -->
      <div class="flex items-center justify-between px-5 py-4 shrink-0 rounded-b-xl"
           :style="{ backgroundColor: 'var(--theme-cell)', borderTop: '1px solid var(--theme-border)' }">
        <button @click="handleReset"
          class="flex items-center gap-1.5 px-3 py-2 text-[13px] font-medium rounded-lg transition-all hover:bg-black/5"
          :style="{ color: 'var(--theme-text-muted)' }">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          恢复默认
        </button>
        <div class="flex gap-2">
          <button @click="handleClose"
            class="w-10 h-10 flex items-center justify-center rounded-xl transition-all hover:bg-black/5"
            :style="{ color: 'var(--theme-text-muted)' }"
            title="取消">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
          <button @click="handleSave"
            class="w-10 h-10 flex items-center justify-center rounded-xl transition-all active:scale-90"
            :style="{
              backgroundColor: 'var(--theme-primary)',
              color: '#ffffff'
            }"
            title="保存">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-panel::-webkit-scrollbar {
  width: 6px;
}
.settings-panel::-webkit-scrollbar-track {
  background: transparent;
}
.settings-panel::-webkit-scrollbar-thumb {
  background: rgba(128, 128, 128, 0.2);
  border-radius: 10px;
}
.settings-panel::-webkit-scrollbar-thumb:hover {
  background: rgba(128, 128, 128, 0.3);
}
</style>