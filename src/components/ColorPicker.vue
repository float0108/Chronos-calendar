<script setup lang="ts">
defineProps<{
  modelValue: string;
  label: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
}>();

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
</script>

<template>
  <div class="color-picker">
    <label class="block text-sm font-medium text-gray-700 mb-2">
      {{ label }}
    </label>
    <div class="flex items-center gap-3">
      <input
        type="color"
        :value="modelValue"
        @input="handleChange"
        class="w-10 h-10 rounded-lg border border-gray-300 cursor-pointer bg-transparent"
      />
      <input
        type="text"
        :value="modelValue"
        @input="handleChange"
        class="flex-1 px-3 py-2 text-sm rounded-lg border border-gray-300 bg-white text-gray-900 focus:outline-none focus:border-blue-500"
        placeholder="#3b82f6"
      />
    </div>
    <div class="flex flex-wrap gap-1.5 mt-2">
      <button
        v-for="color in presetColors"
        :key="color"
        @click="selectPreset(color)"
        class="w-6 h-6 rounded-full border border-gray-200 transition-transform hover:scale-110"
        :style="{ backgroundColor: color }"
        :class="{ 'ring-2 ring-blue-500': color === modelValue }"
      />
    </div>
  </div>
</template>
