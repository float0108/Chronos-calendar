<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue';
import { X, Check, Sun, Moon, RotateCcw, RefreshCw } from 'lucide-vue-next';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { ask } from '@tauri-apps/plugin-dialog';
import type { AppSettings, ThemeMode } from '../types';
import { defaultLightSettings, defaultDarkSettings, extractCommonParts } from '../types/index';
import { hexToRgba, adjustBrightness } from '../utils/color';
import { useFonts } from '../composables/useFonts';
import ColorPicker from './ColorPicker.vue';
import SliderControl from './SliderControl.vue';

// 使用字体缓存
const { loadFonts, cachedFonts, isFontsLoaded } = useFonts();

// 主 Tab: common (公用) / mode (模式)
const activeMainTab = ref<'common' | 'mode'>('common');
const activeTab = ref<ThemeMode>('light');
const localSettings = ref<AppSettings>({ ...defaultLightSettings });
const originalSettings = ref<AppSettings | null>(null);
const hasChanges = ref(false);
const isRefreshingFonts = ref(false);
const fontsLoading = ref(!isFontsLoaded());

// 刷新字体列表
async function refreshFonts() {
  isRefreshingFonts.value = true;
  await loadFonts(true);
  isRefreshingFonts.value = false;
}

// 加载当前设置
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

// 监听设置变化
watch(localSettings, () => {
  if (originalSettings.value) {
    hasChanges.value = JSON.stringify(localSettings.value) !== JSON.stringify(originalSettings.value);
  }
}, { deep: true });

// 应用设置到窗口
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
  // 先加载设置（快速）
  await loadSettings();

  // 确保 DOM 已经根据设置渲染完毕，再显示窗口
  await nextTick();
  requestAnimationFrame(async () => {
    const win = getCurrentWindow();
    await win.show();
    await win.setFocus();
  });

  // 字体异步加载，不阻塞窗口显示
  loadFonts().finally(() => {
    fontsLoading.value = false;
  });
});

async function handleClose() {
  // 检查修改并关闭窗口
  if (hasChanges.value) {
    const shouldClose = await ask('设置已修改但未保存，是否放弃修改？', {
      title: '确认关闭',
      kind: 'warning',
      okLabel: '放弃修改',
      cancelLabel: '取消',
    });

    if (!shouldClose) {
      return;
    }
  }
  getCurrentWindow().close();
}

async function handleSave() {
  const savedSettings = { ...localSettings.value, theme_mode: activeTab.value };

  try {
    localStorage.setItem('chronos_settings', JSON.stringify(savedSettings));
    localStorage.setItem(`chronos_settings_${activeTab.value}`, JSON.stringify(savedSettings));

    // 触发 storage 事件通知主窗口
    window.dispatchEvent(new StorageEvent('storage', {
      key: 'chronos_settings',
      newValue: JSON.stringify(savedSettings),
      url: window.location.href
    }));

    // 同步到另一种模式
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
  <div class="settings-overlay fixed inset-0 z-[9999] flex w-full h-full bg-transparent">
    <div class="settings-panel relative bg-white/70 backdrop-blur-2xl flex flex-col overflow-hidden w-full h-full rounded-none border-0">
      <div class="flex items-center px-6 border-b border-white/40 bg-white/30 z-10 shrink-0" data-tauri-drag-region>
        <button @click="activeMainTab = 'common'"
          :class="['py-4 px-4 text-sm font-medium transition-colors relative border-b-2 -mb-px', activeMainTab === 'common' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-800']">
          公用配置
        </button>
        <button @click="activeMainTab = 'mode'"
          :class="['py-4 px-4 text-sm font-medium transition-colors relative border-b-2 -mb-px', activeMainTab === 'mode' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-800']">
          模式配置
        </button>
        <div class="flex-1" data-tauri-drag-region></div>
        <button @click="handleClose" class="p-1.5 rounded-lg hover:bg-white/50 text-gray-500 hover:text-gray-800 transition-colors" title="关闭">
          <X class="w-5 h-5 pointer-events-none" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto custom-scrollbar">

        <div v-show="activeMainTab === 'common'" class="px-8 py-6 space-y-10">

          <div class="space-y-5">
            <h3 class="text-sm font-bold text-gray-800 uppercase tracking-wider flex items-center gap-2">
              <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span> 排版与字体
            </h3>

            <div class="space-y-4 bg-white/40 rounded-2xl p-4 border border-white/50 shadow-sm">
              <div class="space-y-1.5">
                <label class="text-sm font-medium text-gray-700 ml-1">全局字体</label>
                <div class="flex items-center gap-2">
                  <select v-model="localSettings.font_family"
                    :disabled="fontsLoading"
                    class="flex-1 px-3 py-2 bg-white/60 border border-white/80 rounded-xl text-sm text-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:bg-white transition-all disabled:opacity-50">
                    <template v-if="fontsLoading">
                      <option value="">加载字体中...</option>
                    </template>
                    <template v-else>
                      <option v-for="opt in cachedFonts" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
                    </template>
                  </select>
                  <button @click="refreshFonts"
                    :disabled="isRefreshingFonts || fontsLoading"
                    class="p-2 rounded-xl bg-white/60 border border-white/80 hover:bg-white transition-all disabled:opacity-50"
                    title="刷新字体列表">
                    <RefreshCw :class="['w-4 h-4 text-gray-600 pointer-events-none', (isRefreshingFonts || fontsLoading) ? 'animate-spin' : '']" />
                  </button>
                </div>
              </div>

              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-700">基础字号</label>
                <div class="flex items-center gap-2">
                  <input type="number" v-model.number="localSettings.font_size" min="10" max="32" step="0.5"
                    class="w-20 px-3 py-2 text-sm text-center bg-white/60 border border-white/80 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:bg-white transition-all" />
                  <span class="text-xs text-gray-500 w-4">px</span>
                </div>
              </div>

              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-700">基础字重</label>
                <div class="flex items-center gap-2">
                  <input type="number" v-model.number="localSettings.font_weight" min="100" max="900" step="100"
                    class="w-20 px-3 py-2 text-sm text-center bg-white/60 border border-white/80 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:bg-white transition-all" />
                  <span class="w-4"></span>
                </div>
              </div>
            </div>
          </div>

          <div class="space-y-5">
            <h3 class="text-sm font-bold text-gray-800 uppercase tracking-wider flex items-center gap-2">
              <span class="w-1.5 h-1.5 rounded-full bg-purple-500"></span> 网格与外观
            </h3>

            <div class="space-y-4 bg-white/40 rounded-2xl p-4 border border-white/50 shadow-sm">
              <div class="flex items-center justify-between">
                <div>
                  <label class="text-sm font-medium text-gray-700 cursor-pointer" for="header_style">标题栏应用单元格风格</label>
                  <p class="text-[11px] text-gray-500 mt-0.5">应用单元格背景、边框和透明度到标题栏</p>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" id="header_style" v-model="localSettings.header_cell_style" class="sr-only peer" />
                  <div class="w-11 h-6 bg-gray-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                </label>
              </div>
              <hr class="border-white/40">
              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-700">单元格间距</label>
                <div class="flex items-center gap-2">
                  <input type="number" v-model.number="localSettings.cell_gap" min="0" max="24" step="0.1"
                    class="w-20 px-3 py-1.5 text-sm text-center bg-white/60 border border-white/80 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:bg-white transition-all" />
                  <span class="text-xs text-gray-500 w-4">px</span>
                </div>
              </div>
              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-700">边框粗细</label>
                <div class="flex items-center gap-2">
                  <input type="number" v-model.number="localSettings.cell_border_width" min="0" max="10" step="0.1"
                    class="w-20 px-3 py-1.5 text-sm text-center bg-white/60 border border-white/80 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:bg-white transition-all" />
                  <span class="text-xs text-gray-500 w-4">px</span>
                </div>
              </div>
            </div>
          </div>

          <div class="space-y-5">
            <h3 class="text-sm font-bold text-gray-800 uppercase tracking-wider flex items-center gap-2">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> 系统与日历
            </h3>

            <div class="space-y-4 bg-white/40 rounded-2xl p-4 border border-white/50 shadow-sm">
              <div class="flex items-center justify-between">
                <div>
                  <label class="text-sm font-medium text-gray-700 cursor-pointer" for="autostart">开机自启动</label>
                  <p class="text-[11px] text-gray-500 mt-0.5">启动后应用将在后台静默运行</p>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" id="autostart" v-model="localSettings.autostart" class="sr-only peer" />
                  <div class="w-11 h-6 bg-gray-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500"></div>
                </label>
              </div>
              <hr class="border-white/40">
              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-700">每周开始于</label>
                <div class="flex gap-4 bg-white/50 p-1 rounded-lg border border-white/60">
                  <label class="flex items-center gap-1.5 px-3 py-1 cursor-pointer rounded-md transition-colors" :class="localSettings.week_starts_on === 0 ? 'bg-white shadow-sm' : ''">
                    <input type="radio" :value="0" v-model="localSettings.week_starts_on" class="sr-only" />
                    <span class="text-sm" :class="localSettings.week_starts_on === 0 ? 'text-gray-900 font-medium' : 'text-gray-500'">周日</span>
                  </label>
                  <label class="flex items-center gap-1.5 px-3 py-1 cursor-pointer rounded-md transition-colors" :class="localSettings.week_starts_on === 1 ? 'bg-white shadow-sm' : ''">
                    <input type="radio" :value="1" v-model="localSettings.week_starts_on" class="sr-only" />
                    <span class="text-sm" :class="localSettings.week_starts_on === 1 ? 'text-gray-900 font-medium' : 'text-gray-500'">周一</span>
                  </label>
                </div>
              </div>

              <div class="space-y-3 pt-2">
                <label class="text-sm font-medium text-gray-700">显示模式</label>
                <div class="grid grid-cols-2 gap-3">
                  <label class="flex flex-col items-center justify-center gap-2 p-3 border rounded-xl cursor-pointer transition-all"
                         :class="localSettings.display_mode === 'month' ? 'border-blue-400 bg-white shadow-sm ring-1 ring-blue-400/20' : 'border-white/60 bg-white/40 hover:bg-white/60'">
                    <input type="radio" value="month" v-model="localSettings.display_mode" class="sr-only" />
                    <span class="text-sm font-medium text-gray-800">整月显示</span>
                  </label>
                  <label class="flex flex-col items-center justify-center gap-2 p-3 border rounded-xl cursor-pointer transition-all"
                         :class="localSettings.display_mode === 'floating_weeks' ? 'border-blue-400 bg-white shadow-sm ring-1 ring-blue-400/20' : 'border-white/60 bg-white/40 hover:bg-white/60'">
                    <input type="radio" value="floating_weeks" v-model="localSettings.display_mode" class="sr-only" />
                    <span class="text-sm font-medium text-gray-800">浮动周</span>
                  </label>
                </div>
              </div>

              <div v-if="localSettings.display_mode === 'floating_weeks'" class="flex items-center justify-between pt-2 border-t border-white/40">
                <div>
                  <label class="text-sm font-medium text-gray-700">浮动周数量</label>
                  <p class="text-[10px] text-gray-500 mt-0.5 leading-tight">
                    奇数: 当周居中<br>偶数: 偏向前置周
                  </p>
                </div>
                <div class="flex items-center gap-2">
                  <input type="number" v-model.number="localSettings.floating_weeks_count" min="2" max="10" step="1"
                    class="w-20 px-3 py-1.5 text-sm text-center bg-white/60 border border-white/80 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:bg-white transition-all" />
                  <span class="text-xs text-gray-500 w-4">周</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-show="activeMainTab === 'mode'" class="flex flex-col h-full">
          <div class="px-8 pt-6 pb-2 sticky top-0 z-10 bg-white/10 backdrop-blur-sm">
             <div class="flex p-1.5 gap-2 bg-white/40 rounded-2xl border border-white/50 backdrop-blur-md shadow-sm">
                <button @click="handleSwitchMode('light')"
                  :class="['flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl text-sm font-medium transition-all duration-300', activeTab === 'light' ? 'bg-white text-orange-500 shadow-sm' : 'text-gray-500 hover:text-gray-700 hover:bg-white/50']">
                  <Sun class="w-4 h-4 pointer-events-none" /> 浅色模式
                </button>
                <button @click="handleSwitchMode('dark')"
                  :class="['flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl text-sm font-medium transition-all duration-300', activeTab === 'dark' ? 'bg-slate-800 text-blue-300 shadow-sm' : 'text-gray-500 hover:text-gray-700 hover:bg-white/50']">
                  <Moon class="w-4 h-4 pointer-events-none" /> 深色模式
                </button>
             </div>
          </div>

          <div class="px-8 py-4 space-y-10">
            <div class="space-y-5">
              <h3 class="text-sm font-bold text-gray-800 uppercase tracking-wider flex items-center gap-2">
                <span class="w-1.5 h-1.5 rounded-full bg-rose-400"></span> 核心色彩
              </h3>
              <div class="space-y-4 bg-white/40 rounded-2xl p-5 border border-white/50 shadow-sm">
                <ColorPicker v-model="localSettings.primary_color" label="主题色" />
                <ColorPicker v-model="localSettings.text_color" label="文字颜色" />
                <ColorPicker v-model="localSettings.bg_color" label="窗口背景" :allow-transparent="true" />
                <hr class="border-white/40 my-2">
                <ColorPicker v-model="localSettings.cell_color" label="单元格背景" :allow-transparent="true" />
                <ColorPicker v-model="localSettings.cell_border_color" label="单元格边框" :allow-transparent="true" />
              </div>
            </div>

            <div class="space-y-5 pb-8">
               <h3 class="text-sm font-bold text-gray-800 uppercase tracking-wider flex items-center gap-2">
                <span class="w-1.5 h-1.5 rounded-full bg-cyan-500"></span> 特效与透明度
              </h3>
              <div class="space-y-5 bg-white/40 rounded-2xl p-5 border border-white/50 shadow-sm">
                <div class="flex items-center justify-between pb-2">
                  <div>
                    <h3 class="text-sm font-medium text-gray-800">毛玻璃背景特效</h3>
                    <p class="text-xs text-gray-500 mt-0.5">为应用主窗口开启模糊效果</p>
                  </div>
                  <label class="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" v-model="localSettings.enable_blur" class="sr-only peer" />
                    <div class="w-11 h-6 bg-gray-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-cyan-500"></div>
                  </label>
                </div>
                <hr class="border-white/40">
                <SliderControl v-model="localSettings.bg_opacity" label="窗口背景不透明度" :min="0" :max="100" :step="5" unit="%" />
                <SliderControl v-model="localSettings.cell_opacity" label="单元格背景不透明度" :min="0" :max="100" :step="5" unit="%" />
              </div>
            </div>
          </div>
        </div>

      </div>

      <div class="flex items-center justify-between px-6 py-4 border-t border-white/40 bg-white/30 backdrop-blur-md z-10 shrink-0">
        <button @click="handleReset"
          class="flex items-center gap-1.5 px-3 py-2 text-sm font-medium text-gray-500 hover:text-gray-900 hover:bg-white/50 rounded-xl transition-all">
          <RotateCcw class="w-4 h-4 pointer-events-none" /> 恢复默认
        </button>
        <div class="flex gap-2.5">
          <button @click="handleClose"
            class="p-2.5 rounded-xl font-medium text-gray-600 bg-white/50 hover:bg-white transition-all border border-white/60" title="取消">
            <X class="w-5 h-5 pointer-events-none" />
          </button>
          <button @click="handleSave"
            class="p-2.5 rounded-xl font-medium text-white bg-blue-500 hover:bg-blue-600 transition-all" title="保存">
            <Check class="w-5 h-5 pointer-events-none" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 自定义纤细滚动条以匹配玻璃质感 */
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.1);
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 0, 0, 0.2);
}
</style>