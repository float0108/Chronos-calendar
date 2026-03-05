<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { X, Sun, Moon, RotateCcw } from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, ThemeMode } from '../types';
import { defaultLightSettings, defaultDarkSettings, extractCommonParts } from '../types/index';
import ColorPicker from './ColorPicker.vue';
import SliderControl from './SliderControl.vue';

const props = defineProps<{
  visible: boolean;
  currentSettings: AppSettings | null;
  currentMode: ThemeMode;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', settings: AppSettings): void;
  (e: 'switchMode', mode: ThemeMode): void;
}>();

// 主 Tab: common (公用) / mode (模式)
const activeMainTab = ref<'common' | 'mode'>('common');
const activeTab = ref<ThemeMode>('light');
const localSettings = ref<AppSettings>({ ...defaultLightSettings });

// 系统字体列表
const fontOptions = ref<Array<{ label: string; value: string }>>([
  { label: '系统默认 (System UI)', value: 'system-ui, -apple-system, sans-serif' }
]);

// 加载系统字体
async function loadSystemFonts() {
  try {
    const fonts = await invoke<string[]>('get_system_fonts');
    const systemFonts = fonts.map(font => ({
      label: font,
      value: `"${font}", sans-serif`
    }));
    // 将系统字体添加到默认选项后面
    fontOptions.value = [
      { label: '系统默认 (System UI)', value: 'system-ui, -apple-system, sans-serif' },
      ...systemFonts
    ];
  } catch (error) {
    console.error('Failed to load system fonts:', error);
    // 如果加载失败，保留默认选项
  }
}

watch(() => props.visible, (visible) => {
  if (visible && props.currentSettings) {
    // 补全可能缺失的默认属性
    const defaults = props.currentMode === 'dark' ? defaultDarkSettings : defaultLightSettings;
    localSettings.value = { ...defaults, ...props.currentSettings };
    activeTab.value = props.currentMode;
  }
});

onMounted(() => {
  loadSystemFonts();
});

function handleClose() {
  emit('close');
}

function handleSave() {
  emit('save', { ...localSettings.value, theme_mode: activeTab.value });
  emit('close');
}

function handleSwitchMode(mode: ThemeMode) {
  activeTab.value = mode;

  // 切换模式时，保留当前已修改的【公用配置】
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
  <div v-if="visible" class="settings-overlay fixed inset-0 z-50 flex items-center justify-center">
    <div class="settings-backdrop absolute inset-0 bg-black/50 backdrop-blur-sm" @click="handleClose"></div>

    <div
      class="settings-panel relative bg-white rounded-2xl w-[480px] h-[85vh] max-h-[800px] flex flex-col shadow-2xl overflow-hidden">
      <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 bg-white z-10">
        <h2 class="text-lg font-semibold text-gray-900">设置</h2>
        <button @click="handleClose" class="p-1.5 rounded-lg hover:bg-gray-100 transition-colors">
          <X class="w-5 h-5 text-gray-600" />
        </button>
      </div>

      <div class="flex px-6 border-b border-gray-200 bg-white">
        <button @click="activeMainTab = 'common'"
          :class="['py-3 px-4 text-sm font-medium transition-colors relative border-b-2 -mb-px', activeMainTab === 'common' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700']">
          公用配置
        </button>
        <button @click="activeMainTab = 'mode'"
          :class="['py-3 px-4 text-sm font-medium transition-colors relative border-b-2 -mb-px', activeMainTab === 'mode' ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700']">
          模式配置
        </button>
      </div>

      <div class="flex-1 overflow-y-auto bg-gray-50/50">

        <div v-show="activeMainTab === 'common'" class="px-6 py-5 space-y-8">

          <div class="space-y-6">
            <h3 class="text-sm font-semibold text-gray-900 border-b pb-2">排版与字体</h3>

            <div class="space-y-2">
              <label class="text-sm font-medium text-gray-700">全局字体</label>
              <select v-model="localSettings.font_family"
                class="w-full px-3 py-2 bg-white border border-gray-300 rounded-lg text-sm text-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500">
                <option v-for="opt in fontOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
              </select>
            </div>

            <!-- 基础字号 -->
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-700">基础字号</label>
                <div class="flex items-center gap-2">
                  <input
                    type="number"
                    v-model.number="localSettings.font_size"
                    min="10"
                    max="32"
                    step="0.1"
                    class="w-20 px-2 py-1 text-sm text-right bg-white border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  <span class="text-sm text-gray-500">px</span>
                </div>
              </div>
            </div>
            <!-- 基础字重 -->
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-700">基础字重</label>
                <div class="flex items-center gap-2">
                  <input
                    type="number"
                    v-model.number="localSettings.font_weight"
                    min="100"
                    max="900"
                    step="1"
                    class="w-20 px-2 py-1 text-sm text-right bg-white border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>
            </div>
          </div>

          <div class="space-y-6">
            <h3 class="text-sm font-semibold text-gray-900 border-b pb-2">网格与外观</h3>
            <!-- 单元格间距 -->
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-700">单元格间距</label>
                <div class="flex items-center gap-2">
                  <input
                    type="number"
                    v-model.number="localSettings.cell_gap"
                    min="0"
                    max="24"
                    step="0.1"
                    class="w-20 px-2 py-1 text-sm text-right bg-white border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  <span class="text-sm text-gray-500">px</span>
                </div>
              </div>
            </div>
            <!-- 边框粗细 -->
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-700">边框粗细</label>
                <div class="flex items-center gap-2">
                  <input
                    type="number"
                    v-model.number="localSettings.cell_border_width"
                    min="0"
                    max="10"
                    step="0.1"
                    class="w-20 px-2 py-1 text-sm text-right bg-white border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  <span class="text-sm text-gray-500">px</span>
                </div>
              </div>
            </div>
          </div>

          <div class="space-y-6">
            <h3 class="text-sm font-semibold text-gray-900 border-b pb-2">系统设置</h3>
            <div class="space-y-2">
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="localSettings.autostart"
                  class="w-4 h-4 text-blue-500 rounded border-gray-300 focus:ring-blue-500" />
                <span class="text-sm text-gray-700 font-medium">开机自启动</span>
              </label>
              <p class="text-xs text-gray-500 ml-6">启动后应用将在后台运行</p>
            </div>
          </div>

          <div class="space-y-6">
            <h3 class="text-sm font-semibold text-gray-900 border-b pb-2">日历习惯</h3>
            <div class="space-y-2">
              <label class="text-sm font-medium text-gray-700">每周开始于</label>
              <div class="flex gap-6 mt-2">
                <label class="flex items-center gap-2 cursor-pointer">
                  <input type="radio" :value="0" v-model="localSettings.week_starts_on"
                    class="w-4 h-4 text-blue-500 focus:ring-blue-500" />
                  <span class="text-sm text-gray-700">周日</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                  <input type="radio" :value="1" v-model="localSettings.week_starts_on"
                    class="w-4 h-4 text-blue-500 focus:ring-blue-500" />
                  <span class="text-sm text-gray-700">周一</span>
                </label>
              </div>
            </div>

            <div class="space-y-2">
              <label class="text-sm font-medium text-gray-700">显示模式</label>
              <div class="flex gap-6 mt-2">
                <label class="flex items-center gap-2 cursor-pointer">
                  <input type="radio" value="month" v-model="localSettings.display_mode"
                    class="w-4 h-4 text-blue-500 focus:ring-blue-500" />
                  <span class="text-sm text-gray-700">整月显示</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                  <input type="radio" value="floating_weeks" v-model="localSettings.display_mode"
                    class="w-4 h-4 text-blue-500 focus:ring-blue-500" />
                  <span class="text-sm text-gray-700">浮动周显示</span>
                </label>
              </div>
            </div>

            <div v-if="localSettings.display_mode === 'floating_weeks'" class="space-y-2">
              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-700">浮动周数量</label>
                <div class="flex items-center gap-2">
                  <input
                    type="number"
                    v-model.number="localSettings.floating_weeks_count"
                    min="2"
                    max="10"
                    step="1"
                    class="w-20 px-2 py-1 text-sm text-right bg-white border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  <span class="text-sm text-gray-500">周</span>
                </div>
              </div>
              <p class="text-xs text-gray-500">
                奇数：当天所在周前后各 (n-1)/2 周<br>
                偶数：前 n/2-1 周，后 n/2+1 周
              </p>
            </div>
          </div>

        </div>

        <div v-show="activeMainTab === 'mode'" class="flex flex-col h-full">
          <div class="flex px-6 py-4 gap-2 bg-white border-b border-gray-100 sticky top-0 z-10">
            <button @click="handleSwitchMode('light')"
              :class="['flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg text-sm font-medium transition-colors', activeTab === 'light' ? 'bg-blue-50 text-blue-600 ring-1 ring-blue-500/20' : 'bg-gray-50 text-gray-600 hover:bg-gray-100']">
              <Sun class="w-4 h-4" /> 浅色模式
            </button>
            <button @click="handleSwitchMode('dark')"
              :class="['flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg text-sm font-medium transition-colors', activeTab === 'dark' ? 'bg-slate-800 text-white ring-1 ring-slate-900/20' : 'bg-gray-50 text-gray-600 hover:bg-gray-100']">
              <Moon class="w-4 h-4" /> 深色模式
            </button>
          </div>

          <div class="px-6 py-5 space-y-8">
            <div class="space-y-4">
              <h3 class="text-sm font-semibold text-gray-900 border-b pb-2">颜色配置</h3>
              <ColorPicker v-model="localSettings.primary_color" label="主题色" />
              <ColorPicker v-model="localSettings.text_color" label="文字颜色" />
              <ColorPicker v-model="localSettings.bg_color" label="背景颜色" :allow-transparent="true" />
              <ColorPicker v-model="localSettings.cell_color" label="单元格颜色" :allow-transparent="true" />
              <ColorPicker v-model="localSettings.cell_border_color" label="边框颜色" :allow-transparent="true" />
            </div>

            <div class="space-y-6">
              <div class="flex items-center justify-between border-b pb-2">
                <h3 class="text-sm font-semibold text-gray-900">透明度与特效</h3>
                <label class="flex items-center gap-2 cursor-pointer">
                  <input type="checkbox" v-model="localSettings.enable_blur"
                    class="w-4 h-4 text-blue-500 rounded border-gray-300 focus:ring-blue-500" />
                  <span class="text-sm text-gray-700 font-medium">开启毛玻璃背景</span>
                </label>
              </div>

              <SliderControl v-model="localSettings.bg_opacity" label="窗口背景透明度" :min="0" :max="100" :step="5"
                unit="%" />
              <SliderControl v-model="localSettings.cell_opacity" label="单元格背景透明度" :min="0" :max="100" :step="5"
                unit="%" />
            </div>
          </div>
        </div>

      </div>

      <div class="flex items-center justify-between px-6 py-4 border-t border-gray-200 bg-white z-10">
        <button @click="handleReset"
          class="flex items-center gap-1.5 px-3 py-2 text-sm text-gray-500 hover:text-gray-900 transition-colors">
          <RotateCcw class="w-4 h-4" /> 恢复当前标签页默认
        </button>
        <div class="flex gap-3">
          <button @click="handleClose"
            class="px-5 py-2 text-sm font-medium rounded-lg text-gray-600 hover:bg-gray-100 transition-colors">取消</button>
          <button @click="handleSave"
            class="px-5 py-2 text-sm font-medium rounded-lg bg-blue-500 text-white hover:bg-blue-600 transition-colors shadow-sm shadow-blue-500/20">保存设置</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-overlay {
  animation: fadeIn 0.2s ease;
}

.settings-backdrop {
  animation: fadeIn 0.2s ease;
}

.settings-panel {
  animation: slideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.98);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
</style>