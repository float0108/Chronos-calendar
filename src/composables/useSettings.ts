import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, ThemeMode } from '../types';
import { defaultLightSettings, defaultDarkSettings, extractCommonParts } from '../types';
import { hexToRgba, adjustBrightness } from '../utils/color';

const currentSettings = ref<AppSettings | null>(null);
const currentMode = ref<ThemeMode>('light');

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
    const saved = localStorage.getItem('chronos_settings');
    if (saved) {
      const parsed = JSON.parse(saved);
      // 合并默认配置，防止新增字段为 undefined
      const defaults = parsed.theme_mode === 'dark' ? defaultDarkSettings : defaultLightSettings;
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
    await applySettings();
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

    // 保存当前模式
    localStorage.setItem(`chronos_settings_${settings.theme_mode}`, JSON.stringify(settings));
    localStorage.setItem('chronos_settings', JSON.stringify(settings));

    // 同步公用配置到另一种模式
    const otherMode = settings.theme_mode === 'light' ? 'dark' : 'light';
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

    await applySettings();
  }

  async function applySettings(): Promise<void> {
    if (!currentSettings.value) return;

    const settings = currentSettings.value;
    const root = document.documentElement;

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
    root.style.setProperty('--text-muted', adjustBrightness(textPrimary, 60));
    
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
    
    // 注入 CSS 毛玻璃模糊变量（作为后备）
    root.style.setProperty('--backdrop-blur', settings.enable_blur ? '16px' : '0px');
    
    // 调用原生 API 设置系统级毛玻璃效果
    try {
      await invoke('set_window_vibrancy', { enable: settings.enable_blur });
    } catch (e) {
      // 忽略错误（可能在非 Tauri 环境或调用失败）
    }
    
    // 注入公用配置变量（字体、大小、间距、粗细）
    root.style.setProperty('--font-family-base', settings.font_family);
    root.style.setProperty('--font-size-base', `${settings.font_size}px`);
    root.style.setProperty('--font-weight-base', `${settings.font_weight}`);
    root.style.setProperty('--cell-gap', `${settings.cell_gap}px`);
    root.style.setProperty('--cell-border-width', `${settings.cell_border_width}px`);

    root.setAttribute('data-theme', settings.theme_mode);
    currentMode.value = settings.theme_mode;
  }

  function getSettingsForMode(mode: ThemeMode): AppSettings {
    const saved = localStorage.getItem(`chronos_settings_${mode}`);
    if (saved) {
      const parsed = JSON.parse(saved);
      const defaults = mode === 'light' ? defaultLightSettings : defaultDarkSettings;
      return { ...defaults, ...parsed };
    }
    return mode === 'light' 
      ? { ...defaultLightSettings }
      : { ...defaultDarkSettings };
  }

  async function saveSettingsForMode(mode: ThemeMode, settings: AppSettings): Promise<void> {
    await saveSettings({ ...settings, theme_mode: mode });
  }

  async function switchMode(mode: ThemeMode): Promise<void> {
    const settings = getSettingsForMode(mode);
    currentSettings.value = settings;
    currentMode.value = mode;
    localStorage.setItem('chronos_settings', JSON.stringify(settings));
    await applySettings();
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