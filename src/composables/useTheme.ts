// composables/useTheme.ts
import { ref, computed, onUnmounted } from 'vue';
import { hexToRgba, adjustBrightness } from '../utils/color';
import type { AppSettings } from '../types';
import { defaultLightSettings, defaultDarkSettings } from '../types';

/**
 * 主题管理组合式函数
 *
 * 职责：
 * - 加载和保存主题设置
 * - 计算有效主题（支持系统主题）
 * - 生成主题样式对象
 *
 * @param initialSettings 初始设置
 * @returns 主题相关的响应式数据和方法
 */
export function useTheme(initialSettings: AppSettings) {
  const settings = ref<AppSettings>(initialSettings);
  let themeChangeListener: (() => void) | null = null;

  /**
   * 计算有效主题
   * 支持 system/light/dark 三种模式
   */
  const effectiveTheme = computed(() => {
    if (settings.value.theme_mode === 'system') {
      return (document.documentElement.getAttribute('data-theme') as 'light' | 'dark') || 'light';
    }
    return settings.value.theme_mode;
  });

  /**
   * 计算主题样式对象
   * 缓存 computed 确保只在依赖变化时重算
   */
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
      '--theme-border': s.cell_border_color ||
        (theme === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.05)'),
      '--theme-font-family': s.font_family,
      '--theme-font-size': `${s.font_size}px`,
      'font-family': s.font_family,
      'font-size': `${s.font_size}px`,
    } as Record<string, string>;
  });

  /**
   * 计算单元格样式
   * 用于列表项、编辑框等
   */
  const cellStyle = computed(() => ({
    backgroundColor: 'var(--theme-cell)',
    borderColor: 'var(--theme-border)',
  }));

  /**
   * 从 localStorage 加载主题设置
   */
  function loadSettings() {
    const saved = localStorage.getItem('chronos_settings');
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        const actualTheme = parsed.theme_mode === 'system'
          ? ((document.documentElement.getAttribute('data-theme') as 'light' | 'dark') || 'light')
          : (parsed.theme_mode || 'light');

        // 根据实际主题选择默认值
        const defaults = actualTheme === 'dark' ? defaultDarkSettings : defaultLightSettings;
        settings.value = { ...defaults, ...parsed };
      } catch (error) {
        console.error('Failed to parse theme settings:', error);
        settings.value = initialSettings;
      }
    }
    applyTheme();
  }

  /**
   * 应用主题到 DOM
   */
  function applyTheme() {
    const s = settings.value;
    const root = document.documentElement;
    root.style.setProperty('--primary', s.primary_color);
    root.style.setProperty('--text-primary', s.text_color);
    root.style.setProperty('--text-muted', s.muted_text_color);
  }

  /**
   * 保存主题设置到 localStorage
   */
  function saveSettings(newSettings: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...newSettings };
    localStorage.setItem('chronos_settings', JSON.stringify(settings.value));
    applyTheme();
  }

  /**
   * 监听系统主题变化（仅在 system 模式下）
   */
  function watchSystemTheme() {
    if (settings.value.theme_mode !== 'system') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = () => {
      applyTheme();
    };

    mediaQuery.addEventListener('change', handleChange);
    themeChangeListener = () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }

  /**
   * 清理资源
   */
  onUnmounted(() => {
    if (themeChangeListener) {
      themeChangeListener();
    }
  });

  return {
    settings,
    effectiveTheme,
    themeStyle,
    cellStyle,
    loadSettings,
    applyTheme,
    saveSettings,
    watchSystemTheme,
  };
}