import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import type { AppSettings, ThemeMode } from '../types';
import { defaultLightSettings, defaultDarkSettings, extractCommonParts } from '../types';
import { hexToRgba, adjustBrightness } from '../utils/color';

const currentSettings = ref<AppSettings | null>(null);
const currentMode = ref<ThemeMode>('light');
const systemTheme = ref<'light' | 'dark'>('light');
let systemThemeInitialized = false;

// 解析实际主题（将 system 转换为 light/dark）
const effectiveTheme = computed(() => {
  if (currentMode.value === 'system') {
    return systemTheme.value;
  }
  return currentMode.value;
});

// 应用设置到 DOM
function applySettingsToDom(): void {
  if (!currentSettings.value) return;

  const settings = currentSettings.value;
  const root = document.documentElement;
  const theme = effectiveTheme.value;

  const primary = settings.primary_color;
  const textPrimary = settings.text_color;
  const bgColor = settings.bg_color;
  const bgOpacity = settings.bg_opacity / 100;
  const cellOpacity = settings.cell_opacity / 100;

  // 注入颜色与透明度变量
  root.style.setProperty('--primary', primary);
  root.style.setProperty('--primary-light', hexToRgba(primary, 0.1));
  root.style.setProperty('--primary-light-hover', hexToRgba(primary, 0.2));

  root.style.setProperty('--text-primary', textPrimary);
  root.style.setProperty('--text-secondary', adjustBrightness(textPrimary, 20));
  root.style.setProperty('--text-muted', settings.muted_text_color);

  root.style.setProperty('--glass-bg', hexToRgba(bgColor, bgOpacity));
  root.style.setProperty('--cell-bg', hexToRgba(settings.cell_color, cellOpacity));
  root.style.setProperty('--cell-bg-muted', hexToRgba(settings.cell_color, cellOpacity * 0.6));
  // 单独注入透明度变量，供 color-mix() 使用
  root.style.setProperty('--cell-bg-opacity', String(cellOpacity));

  root.style.setProperty('--hover-bg', hexToRgba('#000000', 0.05));
  root.style.setProperty('--schedule-bg', hexToRgba('#ffffff', cellOpacity + 0.2));
  root.style.setProperty('--schedule-hover', hexToRgba('#ffffff', cellOpacity + 0.4));

  root.style.setProperty('--border-light', hexToRgba('#000000', 0.1));
  // 注入新增的边框颜色
  root.style.setProperty('--cell-border-color', settings.cell_border_color || '#d1d5db');

  // 注入 CSS 毛玻璃模糊变量
  root.style.setProperty('--backdrop-blur', settings.enable_blur ? '16px' : '0px');

  // 注入标题栏样式变量
  if (settings.header_cell_style) {
    // 标题栏应用单元格风格
    root.style.setProperty('--header-bg', 'var(--cell-bg)');
    root.style.setProperty('--header-border-color', 'var(--cell-border-color)');
  } else {
    // 标题栏使用默认样式（透明）
    root.style.setProperty('--header-bg', 'transparent');
    root.style.setProperty('--header-border-color', settings.cell_border_color || '#d1d5db');
  }

  // 注入公用配置变量（字体、大小、间距、粗细）
  root.style.setProperty('--font-family-base', settings.font_family);
  root.style.setProperty('--font-size-base', `${settings.font_size}px`);
  root.style.setProperty('--font-weight-base', `${settings.font_weight}`);
  root.style.setProperty('--cell-gap', `${settings.cell_gap}px`);
  root.style.setProperty('--cell-border-width', `${settings.cell_border_width}px`);
  root.style.setProperty('--cell-border-style', settings.cell_border_style || 'solid');
  root.style.setProperty('--cell-border-dash-interval', `${settings.cell_border_dash_interval || 4}px`);

  root.setAttribute('data-theme', theme);
  currentMode.value = settings.theme_mode;
}

// 初始化系统主题检测
async function initSystemThemeDetection() {
  if (systemThemeInitialized) return;

  try {
    const win = getCurrentWindow();
    const theme = await win.theme();
    systemTheme.value = theme || 'light';
    systemThemeInitialized = true;

    // 监听系统主题变化
    await listen<'light' | 'dark'>('tauri://theme-changed', (event) => {
      systemTheme.value = event.payload || 'light';
      // 如果当前是 system 模式，重新应用设置
      if (currentMode.value === 'system' && currentSettings.value) {
        applySettingsToDom();
      }
    });
  } catch (error) {
    console.error('Failed to init system theme:', error);
    // Fallback
    if (typeof window !== 'undefined') {
      systemTheme.value = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
  }
}

// 初始化自启动状态
async function initAutostart(): Promise<boolean> {
  try {
    const enabled = await invoke<boolean>('get_autostart_status');
    return enabled;
  } catch (error) {
    console.error('Failed to get autostart status:', error);
    return false;
  }
}

// 设置自启动
async function setAutostart(enable: boolean): Promise<void> {
  try {
    await invoke('set_autostart', { enable });
  } catch (error) {
    console.error('Failed to set autostart:', error);
    throw error;
  }
}

export function useSettings() {
  async function initSettings(): Promise<void> {
    // 初始化系统主题检测
    await initSystemThemeDetection();

    const saved = localStorage.getItem('chronos_settings');
    if (saved) {
      const parsed = JSON.parse(saved);
      // 根据 theme_mode 和实际主题加载对应默认值
      const actualTheme = parsed.theme_mode === 'system'
        ? systemTheme.value
        : (parsed.theme_mode || 'light');
      const defaults = actualTheme === 'dark' ? defaultDarkSettings : defaultLightSettings;
      currentSettings.value = { ...defaults, ...parsed };

      // 同步自启动状态（从系统读取）
      try {
        const autostartEnabled = await initAutostart();
        if (currentSettings.value) {
          currentSettings.value.autostart = autostartEnabled;
        }
      } catch (error) {
        console.error('Failed to sync autostart status:', error);
      }

      currentMode.value = currentSettings.value?.theme_mode || 'light';
    } else {
      currentSettings.value = { ...defaultLightSettings };
    }
    applySettingsToDom();
  }

  async function saveSettings(settings: AppSettings): Promise<void> {
    currentSettings.value = { ...settings };
    currentMode.value = settings.theme_mode;

    // 应用自启动设置
    try {
      await setAutostart(settings.autostart);
    } catch (error) {
      console.error('Failed to apply autostart setting:', error);
    }

    // 提取公用配置部分
    const commonParts = extractCommonParts(settings);

    // 获取实际主题
    const actualMode = settings.theme_mode === 'system' ? systemTheme.value : settings.theme_mode;

    // 保存当前设置到实际主题对应的存储
    localStorage.setItem(`chronos_settings_${actualMode}`, JSON.stringify(settings));
    localStorage.setItem('chronos_settings', JSON.stringify(settings));

    // 同步公用配置到另一种模式
    const otherMode = actualMode === 'light' ? 'dark' : 'light';
    const otherSaved = localStorage.getItem(`chronos_settings_${otherMode}`);
    if (otherSaved) {
      const otherSettings = JSON.parse(otherSaved);
      const updatedOther = { ...otherSettings, ...commonParts };
      localStorage.setItem(`chronos_settings_${otherMode}`, JSON.stringify(updatedOther));
    } else {
      // 如果另一种模式还没保存过，用默认值加上当前的公用配置进行初始化
      const otherDefaults = otherMode === 'light' ? defaultLightSettings : defaultDarkSettings;
      localStorage.setItem(`chronos_settings_${otherMode}`, JSON.stringify({ ...otherDefaults, ...commonParts }));
    }

    applySettingsToDom();
  }

  async function applySettings(): Promise<void> {
    applySettingsToDom();
  }

  function getSettingsForMode(mode: ThemeMode): AppSettings {
    // 对于 system 模式，返回当前系统主题对应的设置
    const actualMode = mode === 'system' ? systemTheme.value : mode;

    const saved = localStorage.getItem(`chronos_settings_${actualMode}`);
    if (saved) {
      const parsed = JSON.parse(saved);
      const defaults = actualMode === 'light' ? defaultLightSettings : defaultDarkSettings;
      return { ...defaults, ...parsed, theme_mode: mode };
    }
    const defaults = actualMode === 'light' ? defaultLightSettings : defaultDarkSettings;
    return { ...defaults, theme_mode: mode };
  }

  async function saveSettingsForMode(mode: ThemeMode, settings: AppSettings): Promise<void> {
    await saveSettings({ ...settings, theme_mode: mode });
  }

  async function switchMode(mode: ThemeMode): Promise<void> {
    const settings = getSettingsForMode(mode);
    currentSettings.value = settings;
    currentMode.value = mode;
    localStorage.setItem('chronos_settings', JSON.stringify(settings));
    applySettingsToDom();
  }

  function getSetting<K extends keyof AppSettings>(key: K): AppSettings[K] | undefined {
    if (currentSettings.value) {
      return currentSettings.value[key];
    }

    const saved = localStorage.getItem('chronos_settings');
    if (saved) {
      const parsed = JSON.parse(saved);
      return parsed[key];
    }

    return undefined;
  }

  function getAllSettings(): AppSettings | null {
    if (currentSettings.value) {
      return currentSettings.value;
    }

    const saved = localStorage.getItem('chronos_settings');
    if (saved) {
      return JSON.parse(saved);
    }

    return null;
  }

  return {
    currentSettings,
    currentMode,
    effectiveTheme,
    systemTheme,
    initSettings,
    saveSettings,
    applySettings,
    getSettingsForMode,
    saveSettingsForMode,
    switchMode,
    getSetting,
    getAllSettings,
  };
}