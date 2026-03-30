<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick, computed } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { ask } from '@tauri-apps/plugin-dialog';
import type { AppSettings, ThemeMode } from '../types';
import { defaultLightSettings, defaultDarkSettings, extractCommonParts } from '../types/index';
import { hexToRgba, adjustBrightness } from '../utils/color';
import { useFonts } from '../composables/useFonts';
import CommonSettings from './CommonSettings.vue';
import ModeSettings from './ModeSettings.vue';
import PageSettings from './PageSettings.vue';

const { loadFonts, isFontsLoaded } = useFonts();

const activeMainTab = ref<'common' | 'mode' | 'page'>('common');
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
  root.style.setProperty('--cell-border-style', s.cell_border_style || 'solid');
  root.style.setProperty('--cell-border-dash-interval', `${s.cell_border_dash_interval || 4}px`);

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
  // 添加键盘快捷键
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});

async function handleKeydown(e: KeyboardEvent) {
  // ESC 关闭窗口
  if (e.key === 'Escape') {
    e.preventDefault();
    await handleClose();
    return;
  }
  // Ctrl+S 保存并关闭
  if ((e.ctrlKey || e.metaKey) && (e.key === 's' || e.key === 'S')) {
    e.preventDefault();
    await handleSave();
    return;
  }
}

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
    <div class="settings-panel relative flex flex-col overflow-hidden"
         :class="{ 'dark-mode': localSettings.theme_mode === 'dark', 'light-mode': localSettings.theme_mode === 'light' }">
      <div class="flex flex-1 overflow-hidden">
        <!-- 左侧导航栏 -->
        <div class="w-48 shrink-0 flex flex-col settings-sidebar">
          <!-- 标题区 -->
          <div class="flex items-center gap-2 px-4 py-4 shrink-0" data-tauri-drag-region>
            <span class="w-1.5 h-1.5 rounded-full animate-pulse" :style="{ backgroundColor: 'var(--theme-primary)' }"></span>
            <h1 class="text-[15px] font-bold tracking-tight settings-text">设置</h1>
          </div>

          <!-- 导航菜单 -->
          <nav class="flex-1 px-3 py-2 space-y-1">
            <button @click="activeMainTab = 'common'"
              class="w-full flex items-center gap-3 px-3 py-2.5 text-[13px] font-medium rounded-lg transition-all text-left"
              :class="activeMainTab === 'common' ? 'settings-nav-active' : 'settings-nav-inactive'">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
              公用配置
            </button>
            <button @click="activeMainTab = 'mode'"
              class="w-full flex items-center gap-3 px-3 py-2.5 text-[13px] font-medium rounded-lg transition-all text-left"
              :class="activeMainTab === 'mode' ? 'settings-nav-active' : 'settings-nav-inactive'">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
              </svg>
              模式配置
            </button>
            <button @click="activeMainTab = 'page'"
              class="w-full flex items-center gap-3 px-3 py-2.5 text-[13px] font-medium rounded-lg transition-all text-left"
              :class="activeMainTab === 'page' ? 'settings-nav-active' : 'settings-nav-inactive'">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2" />
              </svg>
              页面设置
            </button>
          </nav>

          <!-- 底部操作区 -->
          <div class="px-3 py-3 space-y-2 settings-sidebar-footer">
            <button @click="handleReset"
              class="w-full flex items-center gap-2 px-3 py-2 text-[12px] font-medium rounded-lg transition-all hover:bg-black/5 settings-nav-inactive">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              恢复默认
            </button>
            <div class="flex gap-2">
              <button @click="handleClose"
                class="flex-1 h-9 flex items-center justify-center rounded-lg transition-all hover:bg-black/5 settings-nav-inactive"
                title="取消">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
              <button @click="handleSave"
                class="flex-1 h-9 flex items-center justify-center rounded-lg transition-all active:scale-90"
                :style="{
                  backgroundColor: 'var(--theme-primary)',
                  color: '#ffffff'
                }"
                title="保存">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
                </svg>
              </button>
            </div>
          </div>
        </div>

        <!-- 右侧内容区 -->
        <div class="flex-1 overflow-hidden flex flex-col settings-content">
          <!-- 顶部标题栏 -->
          <div class="flex items-center justify-between px-5 py-4 shrink-0 settings-content-header" data-tauri-drag-region>
            <h2 class="text-[14px] font-semibold settings-text">{{ activeMainTab === 'common' ? '公用配置' : activeMainTab === 'mode' ? '模式配置' : '页面设置' }}</h2>
            <button @click="handleClose"
              class="w-8 h-8 flex items-center justify-center rounded-lg transition-all hover:bg-black/5 dark:hover:bg-white/10"
              title="关闭">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- 内容 -->
          <div class="flex-1 overflow-y-auto px-5 pb-5">
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

            <PageSettings
              v-show="activeMainTab === 'page'"
              :settings="localSettings"
              @update:settings="localSettings = $event"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-panel {
  width: 100%;
  height: 100%;
  backdrop-filter: blur(16px);
}

/* 亮色模式：白色背景，黑色文字 */
.settings-panel.light-mode {
  background-color: #ffffff;
  color: #000000;
}

/* 暗色模式：黑色背景，白色文字 */
.settings-panel.dark-mode {
  background-color: #000000;
  color: #ffffff;
}

/* 文字样式 */
.settings-panel.light-mode .settings-text {
  color: #000000;
}

.settings-panel.dark-mode .settings-text {
  color: #ffffff;
}

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

/* 侧边栏样式 - 亮色模式 */
.settings-panel.light-mode .settings-sidebar {
  background-color: #f3f4f6;
  border-right: 1px solid #e5e7eb;
}

/* 侧边栏样式 - 暗色模式 */
.settings-panel.dark-mode .settings-sidebar {
  background-color: #111827;
  border-right: 1px solid #374151;
}

/* 导航项样式 - 亮色模式 */
.settings-panel.light-mode .settings-nav-active {
  background-color: #ffffff;
  color: #000000;
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
}

.settings-panel.light-mode .settings-nav-inactive {
  color: #6b7280;
}

.settings-panel.light-mode .settings-nav-inactive:hover {
  background-color: rgba(0,0,0,0.05);
  color: #000000;
}

/* 导航项样式 - 暗色模式 */
.settings-panel.dark-mode .settings-nav-active {
  background-color: #374151;
  color: #ffffff;
  box-shadow: 0 1px 3px rgba(0,0,0,0.3);
}

.settings-panel.dark-mode .settings-nav-inactive {
  color: #9ca3af;
}

.settings-panel.dark-mode .settings-nav-inactive:hover {
  background-color: rgba(255,255,255,0.1);
  color: #ffffff;
}

/* 内容区域样式 - 亮色模式 */
.settings-panel.light-mode .settings-content {
  background-color: #ffffff;
}

.settings-panel.light-mode .settings-content-header {
  border-bottom: 1px solid #e5e7eb;
}

/* 内容区域样式 - 暗色模式 */
.settings-panel.dark-mode .settings-content {
  background-color: #000000;
}

.settings-panel.dark-mode .settings-content-header {
  border-bottom: 1px solid #374151;
}

/* 侧边栏底部区域 - 亮色模式 */
.settings-panel.light-mode .settings-sidebar-footer {
  border-top: 1px solid #e5e7eb;
}

/* 侧边栏底部区域 - 暗色模式 */
.settings-panel.dark-mode .settings-sidebar-footer {
  border-top: 1px solid #374151;
}
</style>