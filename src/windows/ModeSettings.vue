<script setup lang="ts">
import { computed, provide } from 'vue';
import { Sun, Moon } from 'lucide-vue-next';
import type { AppSettings, ThemeMode } from '../types';
import ColorPicker from '../components/ui/ColorPicker.vue';
import SliderControl from '../components/ui/SliderControl.vue';
import { hexToRgba } from '../utils/color';

const props = defineProps<{
  settings: AppSettings;
  activeTab: ThemeMode;
}>();

const emit = defineEmits<{
  (e: 'update:settings', value: AppSettings): void;
  (e: 'switchMode', mode: ThemeMode): void;
}>();

function updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
  emit('update:settings', { ...props.settings, [key]: value });
}

// 计算主题相关样式
const themeColors = computed(() => ({
  primary: props.settings.primary_color,
  text: props.settings.text_color,
  textMuted: props.settings.muted_text_color,
  bg: props.settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.03)' : 'rgba(0,0,0,0.02)',
  border: props.settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.04)',
}));

// 单元格样式（用于信息面板）
const cellStyle = computed(() => {
  const cellOpacity = props.settings.cell_opacity / 100;
  return {
    backgroundColor: hexToRgba(props.settings.cell_color, cellOpacity),
    border: `${props.settings.cell_border_width}px solid ${props.settings.cell_border_color || (props.settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.06)')}`,
  };
});

// 提供给子组件使用
provide('themeColors', themeColors);
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Mode Switcher -->
    <div class="px-5 py-3"
         :style="{ borderBottom: '1px solid ' + themeColors.border }">
      <div class="flex p-1 gap-1 rounded-xl" :style="{ backgroundColor: themeColors.bg }">
        <button @click="emit('switchMode', 'light')"
          class="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg text-sm font-medium transition-all"
          :style="activeTab === 'light' ? {
            backgroundColor: 'rgba(255,255,255,0.9)',
            color: '#f97316',
            boxShadow: '0 1px 3px rgba(0,0,0,0.1)'
          } : { color: themeColors.textMuted }">
          <Sun class="w-4 h-4 pointer-events-none" /> 浅色模式
        </button>
        <button @click="emit('switchMode', 'dark')"
          class="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg text-sm font-medium transition-all"
          :style="activeTab === 'dark' ? {
            backgroundColor: 'rgb(30 41 59)',
            color: '#7dd3fc',
            boxShadow: '0 1px 3px rgba(0,0,0,0.2)'
          } : { color: themeColors.textMuted }">
          <Moon class="w-4 h-4 pointer-events-none" /> 深色模式
        </button>
      </div>
    </div>

    <div class="p-5 space-y-6">
      <!-- 核心色彩 -->
      <div class="space-y-3">
        <h3 class="text-sm font-semibold flex items-center gap-2" :style="{ color: themeColors.text }">
          <span class="w-1 h-1 rounded-full" :style="{ backgroundColor: themeColors.primary }"></span> 核心色彩
        </h3>
        <div class="space-y-3 rounded-xl p-4" :style="cellStyle">
          <ColorPicker :model-value="settings.primary_color" @update:model-value="updateSetting('primary_color', $event)" label="主题色" />
          <ColorPicker :model-value="settings.text_color" @update:model-value="updateSetting('text_color', $event)" label="文字颜色" />
          <ColorPicker :model-value="settings.muted_text_color" @update:model-value="updateSetting('muted_text_color', $event)" label="副文字颜色" />
          <ColorPicker :model-value="settings.bg_color" @update:model-value="updateSetting('bg_color', $event)" label="窗口背景" :allow-transparent="true" />
          <hr :style="{ borderColor: themeColors.border }">
          <ColorPicker :model-value="settings.cell_color" @update:model-value="updateSetting('cell_color', $event)" label="单元格背景" :allow-transparent="true" />
          <ColorPicker :model-value="settings.cell_border_color" @update:model-value="updateSetting('cell_border_color', $event)" label="单元格边框" :allow-transparent="true" />
        </div>
      </div>

      <!-- 特效与透明度 -->
      <div class="space-y-3 pb-4">
        <h3 class="text-sm font-semibold flex items-center gap-2" :style="{ color: themeColors.text }">
          <span class="w-1 h-1 rounded-full" :style="{ backgroundColor: themeColors.primary }"></span> 特效与透明度
        </h3>
        <div class="space-y-3 rounded-xl p-4" :style="cellStyle">
          <div class="flex items-center justify-between">
            <div>
              <h3 class="text-sm font-medium" :style="{ color: themeColors.text }">毛玻璃背景特效</h3>
              <p class="text-xs mt-0.5" :style="{ color: themeColors.textMuted, opacity: 0.7 }">为应用主窗口开启模糊效果</p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" :checked="settings.enable_blur" @change="updateSetting('enable_blur', !settings.enable_blur)" class="sr-only peer" />
              <div class="w-11 h-6 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"
                   :style="{
                     backgroundColor: settings.enable_blur ? themeColors.primary : (settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.2)' : 'rgba(0,0,0,0.15)')
                   }"></div>
            </label>
          </div>
          <hr :style="{ borderColor: themeColors.border }">
          <SliderControl :model-value="settings.bg_opacity" @update:model-value="updateSetting('bg_opacity', $event)" label="窗口背景不透明度" :min="0" :max="100" :step="5" unit="%" />
          <SliderControl :model-value="settings.cell_opacity" @update:model-value="updateSetting('cell_opacity', $event)" label="单元格背景不透明度" :min="0" :max="100" :step="5" unit="%" />
        </div>
      </div>
    </div>
  </div>
</template>
