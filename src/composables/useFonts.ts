import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface FontOption {
  label: string;
  value: string;
}

const cachedFonts = ref<FontOption[]>([
  { label: '系统默认 (System UI)', value: 'system-ui, -apple-system, sans-serif' }
]);

let fontsLoaded = false;
let loadingPromise: Promise<void> | null = null;

export function useFonts() {
  async function loadFonts(forceReload = false): Promise<void> {
    // 如果已经加载且不强制刷新，直接返回
    if (fontsLoaded && !forceReload) {
      return;
    }

    // 如果正在加载中，等待加载完成
    if (loadingPromise) {
      return loadingPromise;
    }

    loadingPromise = (async () => {
      try {
        const fonts = await invoke<string[]>('get_system_fonts');
        const systemFonts = fonts.map(font => ({
          label: font,
          value: `"${font}", sans-serif`
        }));
        cachedFonts.value = [
          { label: '系统默认 (System UI)', value: 'system-ui, -apple-system, sans-serif' },
          ...systemFonts
        ];
        fontsLoaded = true;
      } catch (error) {
        console.error('Failed to load system fonts:', error);
      } finally {
        loadingPromise = null;
      }
    })();

    return loadingPromise;
  }

  function getFonts(): FontOption[] {
    return cachedFonts.value;
  }

  function isFontsLoaded(): boolean {
    return fontsLoaded;
  }

  return {
    loadFonts,
    getFonts,
    isFontsLoaded,
    cachedFonts
  };
}
