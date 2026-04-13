<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '../types';
import { hexToRgba } from '../utils/color';

const props = defineProps<{
  settings: AppSettings;
}>();

// 应用设置状态
const autostart = ref(false);
const mcpEnabled = ref(true);
const mcpPort = ref(3269);
const mcpRunning = ref(false);
const mcpCurrentPort = ref<number | null>(null);
const loading = ref(true);
const saving = ref(false);

// 计算主题相关样式
const themeColors = computed(() => ({
  primary: props.settings.primary_color,
  text: props.settings.text_color,
  textMuted: props.settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.5)' : 'rgba(0,0,0,0.5)',
  bg: props.settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.03)' : 'rgba(0,0,0,0.02)',
  border: props.settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.04)',
}));

const cellStyle = computed(() => {
  const cellOpacity = props.settings.cell_opacity / 100;
  return {
    backgroundColor: hexToRgba(props.settings.cell_color, cellOpacity),
    border: `${props.settings.cell_border_width}px solid ${props.settings.cell_border_color || (props.settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.06)')}`,
  };
});

// 加载应用设置
async function loadSettings() {
  loading.value = true;
  try {
    const [appSettings, mcpStatus] = await Promise.all([
      invoke<{ autostart: boolean; mcp_enabled: boolean; mcp_port: number }>('get_app_settings'),
      invoke<{ running: boolean; port: number | null }>('get_mcp_status'),
    ]);
    autostart.value = appSettings.autostart;
    mcpEnabled.value = appSettings.mcp_enabled;
    mcpPort.value = appSettings.mcp_port;
    mcpRunning.value = mcpStatus.running;
    mcpCurrentPort.value = mcpStatus.port;
  } catch (error) {
    console.error('Failed to load app settings:', error);
  } finally {
    loading.value = false;
  }
}

// 保存设置
async function handleSave() {
  saving.value = true;
  try {
    await invoke('save_app_settings', {
      autostart: autostart.value,
      mcpEnabled: mcpEnabled.value,
      mcpPort: mcpPort.value,
    });
    // 更新 MCP 状态
    mcpRunning.value = mcpEnabled.value;
    mcpCurrentPort.value = mcpEnabled.value ? mcpPort.value : null;
  } catch (error) {
    console.error('Failed to save app settings:', error);
  } finally {
    saving.value = false;
  }
}

onMounted(loadSettings);
</script>

<template>
  <div class="p-5 space-y-6">
    <!-- 加载状态 -->
    <div v-if="loading" class="flex items-center justify-center py-8">
      <div class="animate-spin w-6 h-6 border-2 rounded-full"
           :style="{ borderColor: themeColors.border, borderTopColor: themeColors.primary }"></div>
    </div>

    <template v-else>
      <!-- 自启动设置 -->
      <div class="space-y-3">
        <h3 class="text-sm font-semibold flex items-center gap-2" :style="{ color: themeColors.text }">
          <span class="w-1 h-1 rounded-full" :style="{ backgroundColor: themeColors.primary }"></span> 自启动设置
        </h3>

        <div class="space-y-3 rounded-xl p-4" :style="cellStyle">
          <div class="flex items-center justify-between">
            <div>
              <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">开机自启动</label>
              <p class="text-xs mt-0.5" :style="{ color: themeColors.textMuted, opacity: 0.7 }">启动后应用将在后台静默运行</p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" v-model="autostart" class="sr-only peer" />
              <div class="w-11 h-6 rounded-full peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"
                   :style="{
                     backgroundColor: autostart ? themeColors.primary : (props.settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.2)' : 'rgba(0,0,0,0.15)')
                   }"></div>
            </label>
          </div>
        </div>
      </div>

      <!-- MCP 设置 -->
      <div class="space-y-3">
        <h3 class="text-sm font-semibold flex items-center gap-2" :style="{ color: themeColors.text }">
          <span class="w-1 h-1 rounded-full" :style="{ backgroundColor: themeColors.primary }"></span> MCP 服务设置
        </h3>

        <div class="space-y-3 rounded-xl p-4" :style="cellStyle">
          <div class="flex items-center justify-between">
            <div>
              <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">启用 MCP 服务</label>
              <p class="text-xs mt-0.5" :style="{ color: themeColors.textMuted, opacity: 0.7 }">允许 AI 助手通过 MCP 协议访问日历数据</p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" v-model="mcpEnabled" class="sr-only peer" />
              <div class="w-11 h-6 rounded-full peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"
                   :style="{
                     backgroundColor: mcpEnabled ? themeColors.primary : (props.settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.2)' : 'rgba(0,0,0,0.15)')
                   }"></div>
            </label>
          </div>

          <hr :style="{ borderColor: themeColors.border }">

          <div class="flex items-center justify-between py-1">
            <div>
              <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">服务端口</label>
              <p class="text-xs mt-0.5" :style="{ color: themeColors.textMuted, opacity: 0.7 }">MCP 服务的监听端口</p>
            </div>
            <div class="flex items-center gap-2">
              <input type="number" v-model="mcpPort" min="1024" max="65535" step="1"
                :disabled="!mcpEnabled"
                class="w-20 px-2.5 py-1.5 text-sm text-center rounded-lg focus:outline-none focus:ring-2 transition-all disabled:opacity-50"
                :style="{
                  backgroundColor: props.settings.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(255,255,255,0.8)',
                  border: '1px solid ' + themeColors.border,
                  color: themeColors.text
                }" />
            </div>
          </div>

          <!-- MCP 状态显示 -->
          <div class="flex items-center justify-between py-1">
            <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">服务状态</label>
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full"
                    :style="{ backgroundColor: mcpRunning ? '#22c55e' : '#ef4444' }"></span>
              <span class="text-sm" :style="{ color: themeColors.textMuted }">
                {{ mcpRunning ? `运行中 (${mcpCurrentPort})` : '已停止' }}
              </span>
            </div>
          </div>

          <!-- 保存按钮 -->
          <div class="pt-2">
            <button @click="handleSave"
              :disabled="saving"
              class="w-full py-2 px-4 rounded-lg text-sm font-medium transition-all disabled:opacity-50"
              :style="{
                backgroundColor: themeColors.primary,
                color: '#ffffff'
              }">
              {{ saving ? '保存中...' : '保存设置' }}
            </button>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
