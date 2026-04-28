import { computed } from 'vue';
import { hexToRgba, adjustBrightness } from '../utils/color';
import type { AppSettings } from '../types';

export function useThemeStyle(settings: { value: AppSettings }, effectiveTheme: () => 'light' | 'dark') {
  const themeStyle = computed(() => {
    const s = settings.value;
    const bgOpacity = s.bg_opacity / 100;
    const cellOpacity = s.cell_opacity / 100;
    const theme = effectiveTheme();
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

  const panelStyle = computed(() => {
    const s = settings.value;
    return {
      backgroundColor: `var(--theme-bg)`,
      border: '1px solid var(--theme-border)',
      backdropFilter: s.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
      WebkitBackdropFilter: s.enable_blur ? 'blur(20px) saturate(180%)' : 'none',
    };
  });

  const cellStyle = computed(() => ({
    backgroundColor: 'var(--theme-cell)',
    border: '1px solid var(--theme-border)',
  }));

  return { themeStyle, panelStyle, cellStyle };
}
