<script setup lang="ts">
import { computed } from 'vue';

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

const displayValue = computed(() => {
  if (props.modelValue === undefined || isNaN(props.modelValue)) {
    return props.min;
  }
  return props.modelValue;
});

function handleChange(event: Event) {
  emit('update:modelValue', Number((event.target as HTMLInputElement).value));
}
</script>

<template>
  <div class="slider-control">
    <div class="flex items-center justify-between mb-2">
      <label class="text-sm font-medium text-gray-700">
        {{ label }}
      </label>
      <span class="text-sm text-gray-900 font-mono">
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
      class="w-full h-2 rounded-lg appearance-none cursor-pointer bg-gray-200"
    />
  </div>
</template>

<style scoped>
input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  border: 2px solid white;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
}

input[type="range"]::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  border: 2px solid white;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
}
</style>
