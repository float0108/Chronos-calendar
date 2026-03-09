<script setup lang="ts">
import { inject } from 'vue';

defineProps<{
  modelValue: string;
  label: string;
  allowTransparent?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
}>();

// 从父组件注入主题颜色（如果可用）
const themeColors = inject<{ primary: string; text: string; textMuted: string; border: string; bg: string }>('themeColors', {
  primary: '#3b82f6',
  text: '#1f2937',
  textMuted: '#6b7280',
  border: 'rgba(0,0,0,0.1)',
  bg: 'rgba(0,0,0,0.02)'
});

const presetColors = [
  '#3b82f6', '#60a5fa', '#2563eb',
  '#ef4444', '#f87171', '#dc2626',
  '#22c55e', '#4ade80', '#16a34a',
  '#f59e0b', '#fbbf24', '#d97706',
  '#8b5cf6', '#a78bfa', '#7c3aed',
  '#ec4899', '#f472b6', '#db2777',
  '#14b8a6', '#2dd4bf', '#0d9488',
  '#6b7280', '#9ca3af', '#4b5563',
  '#000000', '#ffffff',
];

function handleChange(event: Event) {
  emit('update:modelValue', (event.target as HTMLInputElement).value);
}

function selectPreset(color: string) {
  emit('update:modelValue', color);
}

function selectTransparent() {
  emit('update:modelValue', 'transparent');
}
</script>

<template>
  <div class="color-picker">
    <label class="block text-[13px] font-medium mb-2" :style="{ color: themeColors.textMuted }">
      {{ label }}
    </label>
    <div class="flex items-center gap-3">
      <input
        type="color"
        :value="modelValue === 'transparent' ? '#ffffff' : modelValue"
        @input="handleChange"
        class="w-10 h-10 rounded-lg cursor-pointer bg-transparent"
        :class="{ 'opacity-50': modelValue === 'transparent' }"
        :style="{ border: '1px solid ' + themeColors.border }"
      />
      <input
        type="text"
        :value="modelValue"
        @input="handleChange"
        class="flex-1 px-3 py-2 text-[13px] rounded-lg focus:outline-none"
        :style="{
          border: '1px solid ' + themeColors.border,
          backgroundColor: themeColors.bg,
          color: themeColors.text
        }"
        placeholder="#3b82f6 或 transparent"
      />
    </div>
    <div class="flex flex-wrap gap-1.5 mt-2">
      <!-- 无色按钮 -->
      <button
        v-if="allowTransparent"
        @click="selectTransparent"
        class="w-6 h-6 rounded-full border-2 border-dashed transition-transform hover:scale-110 flex items-center justify-center"
        :style="{
          borderColor: modelValue === 'transparent' ? themeColors.primary : themeColors.border,
          boxShadow: modelValue === 'transparent' ? `0 0 0 2px ${themeColors.primary}33` : 'none'
        }"
        title="无色（透明）"
      >
        <svg class="w-4 h-4" :style="{ color: themeColors.textMuted }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="4" y1="4" x2="20" y2="20" />
        </svg>
      </button>
      <!-- 预设颜色按钮 -->
      <button
        v-for="color in presetColors"
        :key="color"
        @click="selectPreset(color)"
        class="w-6 h-6 rounded-full border transition-transform hover:scale-110"
        :style="{
          backgroundColor: color,
          borderColor: color === '#ffffff' ? themeColors.border : 'transparent',
          boxShadow: color === modelValue ? `0 0 0 2px ${themeColors.primary}` : 'none'
        }"
      />
    </div>
  </div>
</template>
