<script setup lang="ts">
import { computed, inject } from 'vue';

const props = defineProps<{
  modelValue?: number;
  label: string;
  min: number;
  max: number;
  step?: number;
  unit?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void;
}>();

// 从父组件注入主题颜色
const themeColors = inject<{ primary: string; text: string; textMuted: string; border: string; bg: string }>('themeColors', {
  primary: '#3b82f6',
  text: '#1f2937',
  textMuted: '#6b7280',
  border: 'rgba(0,0,0,0.1)',
  bg: 'rgba(0,0,0,0.02)'
});

const displayValue = computed(() => {
  if (props.modelValue === undefined || isNaN(props.modelValue)) {
    return props.min;
  }
  return props.modelValue;
});

function handleChange(event: Event) {
  emit('update:modelValue', Number((event.target as HTMLInputElement).value));
}

// 计算进度条样式
const sliderStyle = computed(() => {
  const percent = ((displayValue.value - props.min) / (props.max - props.min)) * 100;
  return {
    '--thumb-color': themeColors.primary,
    '--track-bg': themeColors.bg,
    '--progress-percent': `${percent}%`,
    background: `linear-gradient(to right, ${themeColors.primary} ${percent}%, ${themeColors.bg} ${percent}%)`
  };
});
</script>

<template>
  <div class="slider-control">
    <div class="flex items-center justify-between mb-2">
      <label class="text-sm font-medium" :style="{ color: themeColors.textMuted }">
        {{ label }}
      </label>
      <span class="text-sm font-mono" :style="{ color: themeColors.text }">
        {{ displayValue }}{{ unit || '' }}
      </span>
    </div>
    <input
      type="range"
      :min="min"
      :max="max"
      :step="step || 1"
      :value="displayValue"
      @input="handleChange"
      class="w-full h-2 rounded-lg appearance-none slider-input"
      :style="sliderStyle"
    />
  </div>
</template>

<style scoped>
.slider-input::-webkit-slider-thumb {
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--thumb-color, #3b82f6);
  cursor: pointer;
  border: 2px solid white;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
}

.slider-input::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--thumb-color, #3b82f6);
  cursor: pointer;
  border: 2px solid white;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
}
</style>
