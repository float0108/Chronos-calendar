<script setup lang="ts">
import { computed } from 'vue';
import { RefreshCw } from 'lucide-vue-next';
import type { AppSettings } from '../types';
import { useFonts } from '../composables/useFonts';
import { hexToRgba } from '../utils/color';

const props = defineProps<{
  settings: AppSettings;
  fontsLoading: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:settings', value: AppSettings): void;
  (e: 'refreshFonts'): void;
}>();

const { cachedFonts } = useFonts();

function updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
  emit('update:settings', { ...props.settings, [key]: value });
}

const isRefreshingFonts = defineModel<boolean>('isRefreshingFonts', { default: false });

// 计算主题相关样式
const themeColors = computed(() => ({
  primary: props.settings.primary_color,
  text: props.settings.text_color,
  textMuted: props.settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.5)' : 'rgba(0,0,0,0.5)',
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
</script>

<template>
  <div class="p-5 space-y-6">
    <!-- 排版与字体 -->
    <div class="space-y-3">
      <h3 class="text-sm font-semibold flex items-center gap-2" :style="{ color: themeColors.text }">
        <span class="w-1 h-1 rounded-full" :style="{ backgroundColor: themeColors.primary }"></span> 排版与字体
      </h3>

      <div class="space-y-3 rounded-xl p-4" :style="cellStyle">
        <div class="space-y-1.5">
          <label class="text-sm font-medium ml-0.5" :style="{ color: themeColors.textMuted }">全局字体</label>
          <div class="flex items-center gap-2">
            <select :value="settings.font_family"
              :disabled="fontsLoading"
              @change="updateSetting('font_family', ($event.target as HTMLSelectElement).value)"
              class="flex-1 px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-2 transition-all disabled:opacity-50"
              :style="{
                backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                border: '1px solid ' + themeColors.border,
                color: themeColors.text
              }">
              <template v-if="fontsLoading">
                <option value="">加载字体中...</option>
              </template>
              <template v-else>
                <option v-for="opt in cachedFonts" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
              </template>
            </select>
            <button @click="emit('refreshFonts')"
              :disabled="isRefreshingFonts || fontsLoading"
              class="p-2 rounded-lg transition-all disabled:opacity-50"
              :style="{
                backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                border: '1px solid ' + themeColors.border
              }"
              title="刷新字体列表">
              <RefreshCw :class="['w-4 h-4 pointer-events-none', (isRefreshingFonts || fontsLoading) ? 'animate-spin' : '']" :style="{ color: themeColors.textMuted }" />
            </button>
          </div>
        </div>

        <div class="flex items-center justify-between py-1">
          <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">基础字号</label>
          <div class="flex items-center gap-2">
            <input type="number" :value="settings.font_size" @input="updateSetting('font_size', Number(($event.target as HTMLInputElement).value))" min="10" max="32" step="0.5"
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
          <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">基础字重</label>
          <div class="flex items-center gap-2">
            <input type="number" :value="settings.font_weight" @input="updateSetting('font_weight', Number(($event.target as HTMLInputElement).value))" min="100" max="900" step="100"
              class="w-16 px-2.5 py-1.5 text-sm text-center rounded-lg focus:outline-none focus:ring-2 transition-all"
              :style="{
                backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                border: '1px solid ' + themeColors.border,
                color: themeColors.text
              }" />
            <span class="w-4"></span>
          </div>
        </div>
      </div>
    </div>

    <!-- 系统与日历 -->
    <div class="space-y-3">
      <h3 class="text-sm font-semibold flex items-center gap-2" :style="{ color: themeColors.text }">
        <span class="w-1 h-1 rounded-full" :style="{ backgroundColor: themeColors.primary }"></span> 系统与日历
      </h3>

      <div class="space-y-3 rounded-xl p-4" :style="cellStyle">
        <div class="flex items-center justify-between">
          <div>
            <label class="text-sm font-medium cursor-pointer" :style="{ color: themeColors.textMuted }">开机自启动</label>
            <p class="text-xs mt-0.5" :style="{ color: themeColors.textMuted, opacity: 0.7 }">启动后应用将在后台静默运行</p>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input type="checkbox" :checked="settings.autostart" @change="updateSetting('autostart', !settings.autostart)" class="sr-only peer" />
            <div class="w-11 h-6 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"
                 :style="{
                   backgroundColor: settings.autostart ? themeColors.primary : (settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.2)' : 'rgba(0,0,0,0.15)')
                 }"></div>
          </label>
        </div>
        <hr :style="{ borderColor: themeColors.border }">
        <div class="flex items-center justify-between py-1">
          <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">每周开始于</label>
          <div class="flex gap-1 p-0.5 rounded-lg" :style="{ backgroundColor: themeColors.bg }">
            <label class="flex items-center px-3 py-1 cursor-pointer rounded-md transition-colors"
                   :style="settings.week_starts_on === 0 ? {
                     backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.1)' : 'rgba(255,255,255,0.9)',
                     boxShadow: '0 1px 2px rgba(0,0,0,0.05)'
                   } : {}">
              <input type="radio" :value="0" :checked="settings.week_starts_on === 0" @change="updateSetting('week_starts_on', 0)" class="sr-only" />
              <span class="text-sm" :style="{ color: settings.week_starts_on === 0 ? themeColors.text : themeColors.textMuted, fontWeight: settings.week_starts_on === 0 ? '500' : 'normal' }">周日</span>
            </label>
            <label class="flex items-center px-3 py-1 cursor-pointer rounded-md transition-colors"
                   :style="settings.week_starts_on === 1 ? {
                     backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.1)' : 'rgba(255,255,255,0.9)',
                     boxShadow: '0 1px 2px rgba(0,0,0,0.05)'
                   } : {}">
              <input type="radio" :value="1" :checked="settings.week_starts_on === 1" @change="updateSetting('week_starts_on', 1)" class="sr-only" />
              <span class="text-sm" :style="{ color: settings.week_starts_on === 1 ? themeColors.text : themeColors.textMuted, fontWeight: settings.week_starts_on === 1 ? '500' : 'normal' }">周一</span>
            </label>
          </div>
        </div>

        <div class="space-y-3 pt-1">
          <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">显示模式</label>
          <div class="grid grid-cols-2 gap-2">
            <button type="button"
                   @click="updateSetting('display_mode', 'month')"
                   class="flex items-center justify-center gap-2 p-2.5 border rounded-lg cursor-pointer transition-all text-left"
                   :style="settings.display_mode === 'month' ? {
                     borderColor: themeColors.primary,
                     backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.05)' : 'rgba(0,0,0,0.02)'
                   } : {
                     borderColor: themeColors.border,
                     backgroundColor: 'transparent'
                   }">
              <span class="text-sm font-medium" :style="{ color: themeColors.text }">整月显示</span>
            </button>
            <button type="button"
                   @click="updateSetting('display_mode', 'floating_weeks')"
                   class="flex items-center justify-center gap-2 p-2.5 border rounded-lg cursor-pointer transition-all text-left"
                   :style="settings.display_mode === 'floating_weeks' ? {
                     borderColor: themeColors.primary,
                     backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.05)' : 'rgba(0,0,0,0.02)'
                   } : {
                     borderColor: themeColors.border,
                     backgroundColor: 'transparent'
                   }">
              <span class="text-sm font-medium" :style="{ color: themeColors.text }">浮动周</span>
            </button>
          </div>
        </div>

        <div class="overflow-hidden transition-all duration-200"
             :style="{
               maxHeight: settings.display_mode === 'floating_weeks' ? '100px' : '0',
               opacity: settings.display_mode === 'floating_weeks' ? 1 : 0,
               paddingTop: settings.display_mode === 'floating_weeks' ? '12px' : '0',
               borderTop: settings.display_mode === 'floating_weeks' ? '1px solid ' + themeColors.border : 'none'
             }">
          <div class="flex items-center justify-between">
            <div>
              <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">浮动周数量</label>
              <p class="text-xs mt-0.5 leading-tight" :style="{ color: themeColors.textMuted, opacity: 0.7 }">奇数: 当周居中 / 偶数: 偏向前置周</p>
            </div>
            <div class="flex items-center gap-2">
              <input type="number" :value="settings.floating_weeks_count" @input="updateSetting('floating_weeks_count', Number(($event.target as HTMLInputElement).value))" min="2" max="10" step="1"
                class="w-16 px-2.5 py-1.5 text-sm text-center rounded-lg focus:outline-none focus:ring-2 transition-all"
                :style="{
                  backgroundColor: settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                  border: '1px solid ' + themeColors.border,
                  color: themeColors.text
                }" />
              <span class="text-sm w-4" :style="{ color: themeColors.textMuted }">周</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
