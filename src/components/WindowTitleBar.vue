<script setup lang="ts">
import { X } from 'lucide-vue-next';
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  themeStyle?: Record<string, string>;
  showCloseButton?: boolean;
}>(), {
  showCloseButton: true,
});

const emit = defineEmits<{
  close: [];
  startDrag: [];
}>();

const textColor = computed(() => props.themeStyle?.['--theme-text'] || 'inherit');
</script>

<template>
  <div class="window-title-bar flex items-center gap-2 px-3 py-2.5 shrink-0 select-none group">
    <slot name="left" />

    <div
      class="flex-1 min-w-0 flex justify-center items-center relative h-6"
      @mousedown="(e) => e.target === e.currentTarget && emit('startDrag')"
    >
      <slot />
    </div>

    <slot name="right" />

    <button
      v-if="showCloseButton"
      @click="emit('close')"
      class="close-btn shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
      :style="{ color: textColor }"
    >
      <X class="w-4 h-4" />
    </button>
  </div>
</template>
