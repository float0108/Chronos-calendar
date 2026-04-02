<script setup lang="ts">
import { computed } from 'vue';
import type { AppSettings, BorderStyle } from '../types';
import { hexToRgba } from '../utils/color';

const props = defineProps<{
  settings: AppSettings;
}>();

const emit = defineEmits<{
  (e: 'update:settings', value: AppSettings): void;
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

// 处理宽高占比输入
function handlePercentInput(key: 'desc_dialog_width' | 'desc_dialog_height', value: string) {
  const num = parseInt(value, 10);
  if (isNaN(num)) return;
  const clamped = Math.max(10, Math.min(90, num));
  // 对齐到 5 的倍数
  const rounded = Math.round(clamped / 5) * 5;
  updateSetting(key, rounded);
}

function adjustPercent(key: 'desc_dialog_width' | 'desc_dialog_height', delta: number) {
  const current = props.settings[key] ?? 40;
  const newValue = Math.max(10, Math.min(90, current + delta));
  updateSetting(key, newValue);
}
</script>

<template>
  <div class="p-5 space-y-6">
    <!-- 网格与外观 -->
    <div class="space-y-3">
      <h3 class="text-sm font-semibold flex items-center gap-2" :style="{ color: themeColors.text }">
        <span class="w-1 h-1 rounded-full" :style="{ backgroundColor: themeColors.primary }"></span> 网格与外观
      </h3>

      <div class="space-y-3 rounded-xl p-4" :style="cellStyle">
        <div class="flex items-center justify-between">
          <div>
            <label class="text-sm font-medium cursor-pointer" :style="{ color: themeColors.textMuted }">标题栏应用单元格风格</label>
            <p class="text-xs mt-0.5" :style="{ color: themeColors.textMuted, opacity: 0.7 }">应用单元格背景、边框和透明度到标题栏</p>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input type="checkbox" :checked="settings.header_cell_style" @change="updateSetting('header_cell_style', !settings.header_cell_style)" class="sr-only peer" />
            <div class="w-11 h-6 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"
                 :style="{
                   backgroundColor: settings.header_cell_style ? themeColors.primary : (settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.2)' : 'rgba(0,0,0,0.15)')
                 }"></div>
          </label>
        </div>
        <hr :style="{ borderColor: themeColors.border }">
        <div class="flex items-center justify-between py-1">
          <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">单元格间距</label>
          <div class="flex items-center gap-2">
            <input type="number" :value="settings.cell_gap" @input="updateSetting('cell_gap', Number(($event.target as HTMLInputElement).value))" min="0" max="24" step="0.1"
              class="w-16 px-2.5 py-1.5 text-sm text-center rounded-lg focus:outline-none focus:ring-2 transition-all"
              :style="{
                backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                border: '1px solid ' + themeColors.border,
                color: themeColors.text
              }" />
            <span class="text-sm w-4" :style="{ color: themeColors.textMuted }">px</span>
          </div>
        </div>
        <div class="flex items-center justify-between py-1">
          <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">边框粗细</label>
          <div class="flex items-center gap-2">
            <input type="number" :value="settings.cell_border_width" @input="updateSetting('cell_border_width', Number(($event.target as HTMLInputElement).value))" min="0" max="10" step="0.1"
              class="w-16 px-2.5 py-1.5 text-sm text-center rounded-lg focus:outline-none focus:ring-2 transition-all"
              :style="{
                backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                border: '1px solid ' + themeColors.border,
                color: themeColors.text
              }" />
            <span class="text-sm w-4" :style="{ color: themeColors.textMuted }">px</span>
          </div>
        </div>

        <div class="flex items-center justify-between py-1">
          <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">边框线型</label>
          <select :value="settings.cell_border_style"
            @change="updateSetting('cell_border_style', ($event.target as HTMLSelectElement).value as BorderStyle)"
            class="w-32 px-2.5 py-1.5 text-sm rounded-lg focus:outline-none focus:ring-2 transition-all"
            :style="{
              backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
              border: '1px solid ' + themeColors.border,
              color: themeColors.text
            }">
            <option value="solid">实线 ──</option>
            <option value="dashed">虚线 - -</option>
            <option value="dotted">点线 ···</option>
            <option value="dash-dot">点划线 -·-</option>
            <option value="dash-dot-dot">双点划线 -··</option>
          </select>
        </div>

        <div class="overflow-hidden transition-all duration-200"
             :style="{
               maxHeight: settings.cell_border_style !== 'solid' ? '80px' : '0',
               opacity: settings.cell_border_style !== 'solid' ? 1 : 0,
               paddingTop: settings.cell_border_style !== 'solid' ? '8px' : '0'
             }">
          <div class="flex items-center justify-between py-1">
            <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">线型间隔</label>
            <div class="flex items-center gap-2">
              <input type="number" :value="settings.cell_border_dash_interval" @input="updateSetting('cell_border_dash_interval', Number(($event.target as HTMLInputElement).value))" min="1" max="20" step="0.5"
                class="w-16 px-2.5 py-1.5 text-sm text-center rounded-lg focus:outline-none focus:ring-2 transition-all"
                :style="{
                  backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                  border: '1px solid ' + themeColors.border,
                  color: themeColors.text
                }" />
              <span class="text-sm w-4" :style="{ color: themeColors.textMuted }">px</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 描述窗口 -->
    <div class="space-y-3">
      <h3 class="text-sm font-semibold flex items-center gap-2" :style="{ color: themeColors.text }">
        <span class="w-1 h-1 rounded-full" :style="{ backgroundColor: themeColors.primary }"></span> 描述窗口
      </h3>

      <div class="space-y-3 rounded-xl p-4" :style="cellStyle">
        <p class="text-sm" :style="{ color: themeColors.textMuted }">
          设置日程详情窗口的尺寸占比
        </p>

        <div class="flex items-center justify-between py-1">
          <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">窗口宽度占比</label>
          <div class="flex items-center gap-1">
            <button @click="adjustPercent('desc_dialog_width', -5)"
              class="w-8 h-8 flex items-center justify-center rounded-lg transition-all hover:opacity-80"
              :style="{
                backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                border: '1px solid ' + themeColors.border,
                color: themeColors.text
              }">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
              </svg>
            </button>
            <input type="number"
              :value="settings.desc_dialog_width ?? 40"
              @blur="handlePercentInput('desc_dialog_width', ($event.target as HTMLInputElement).value)"
              min="10" max="90" step="5"
              class="w-16 px-2.5 py-1.5 text-sm text-center rounded-lg focus:outline-none focus:ring-2 transition-all"
              :style="{
                backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                border: '1px solid ' + themeColors.border,
                color: themeColors.text
              }" />
            <button @click="adjustPercent('desc_dialog_width', 5)"
              class="w-8 h-8 flex items-center justify-center rounded-lg transition-all hover:opacity-80"
              :style="{
                backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                border: '1px solid ' + themeColors.border,
                color: themeColors.text
              }">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
            </button>
            <span class="text-sm w-4 ml-1" :style="{ color: themeColors.textMuted }">%</span>
          </div>
        </div>

        <div class="flex items-center justify-between py-1">
          <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">窗口高度占比</label>
          <div class="flex items-center gap-1">
            <button @click="adjustPercent('desc_dialog_height', -5)"
              class="w-8 h-8 flex items-center justify-center rounded-lg transition-all hover:opacity-80"
              :style="{
                backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                border: '1px solid ' + themeColors.border,
                color: themeColors.text
              }">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
              </svg>
            </button>
            <input type="number"
              :value="settings.desc_dialog_height ?? 70"
              @blur="handlePercentInput('desc_dialog_height', ($event.target as HTMLInputElement).value)"
              min="10" max="90" step="5"
              class="w-16 px-2.5 py-1.5 text-sm text-center rounded-lg focus:outline-none focus:ring-2 transition-all"
              :style="{
                backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                border: '1px solid ' + themeColors.border,
                color: themeColors.text
              }" />
            <button @click="adjustPercent('desc_dialog_height', 5)"
              class="w-8 h-8 flex items-center justify-center rounded-lg transition-all hover:opacity-80"
              :style="{
                backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                border: '1px solid ' + themeColors.border,
                color: themeColors.text
              }">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
            </button>
            <span class="text-sm w-4 ml-1" :style="{ color: themeColors.textMuted }">%</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
