import { ref, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';

type SystemTheme = 'light' | 'dark';

const systemTheme = ref<SystemTheme>('light');
const listeners: Array<() => void> = [];

let initialized = false;

async function initSystemTheme() {
  if (initialized) return;

  try {
    const win = getCurrentWindow();
    const theme = await win.theme();
    systemTheme.value = theme || 'light';
    initialized = true;
  } catch (error) {
    console.error('Failed to get system theme:', error);
    // Fallback to media query
    if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
      systemTheme.value = 'dark';
    }
  }
}

export function useSystemTheme() {
  let unlisten: (() => void) | null = null;

  onMounted(async () => {
    await initSystemTheme();

    // 监听系统主题变化
    try {
      unlisten = await listen<SystemTheme>('tauri://theme-changed', (event) => {
        systemTheme.value = event.payload || 'light';
        listeners.forEach(cb => cb());
      });
    } catch (error) {
      console.error('Failed to listen theme change:', error);
    }

    // 备用：监听 CSS media query 变化
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = (e: MediaQueryListEvent) => {
      systemTheme.value = e.matches ? 'dark' : 'light';
      listeners.forEach(cb => cb());
    };
    mediaQuery.addEventListener('change', handleChange);
  });

  onUnmounted(() => {
    if (unlisten) {
      unlisten();
    }
  });

  function onThemeChange(callback: () => void) {
    listeners.push(callback);
    return () => {
      const index = listeners.indexOf(callback);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    };
  }

  return {
    systemTheme,
    onThemeChange,
  };
}
