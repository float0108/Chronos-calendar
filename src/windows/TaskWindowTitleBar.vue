<script setup lang="ts">
import { ChevronLeft, Plus, X } from 'lucide-vue-next';
import { computed } from 'vue';

const props = defineProps<{
  viewMode: 'task' | 'list' | 'detail';
  isEditingTitle: boolean;
  editingTitle: string;
  currentTitle: string;
  isAdding: boolean;
  themeStyle: Record<string, string>;
  titleInputRef?: HTMLInputElement | null;
}>();

const emit = defineEmits<{
  back: [];
  close: [];
  startAdding: [];
  startEditingTitle: [defaultTitle: string];
  saveTitle: [];
  cancelEditingTitle: [];
  'update:editingTitle': [value: string];
  startDrag: [];
}>();

const titleText = computed(() => {
  if (props.viewMode === 'detail') {
    return props.currentTitle;
  }
  return props.currentTitle || 'Task';
});

const textColor = computed(() => props.themeStyle['--theme-text'] || 'inherit');
</script>

<template>
  <div class="title-bar flex items-center justify-between px-3 py-2.5 shrink-0 select-none group">
    <!-- Left button -->
    <div class="w-[60px] flex items-center justify-start">
      <button @click="emit('back')"
        class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
        :style="{ color: textColor }">
        <ChevronLeft class="w-4 h-4" />
      </button>
    </div>

    <!-- Center title -->
    <div class="flex-1 flex justify-center items-center h-6 px-2"
      @mousedown="emit('startDrag')">
      <template v-if="!isEditingTitle">
        <span
          class="text-base font-medium leading-relaxed transition-opacity truncate max-w-[200px] hover:opacity-80 cursor-pointer"
          @click="emit('startEditingTitle', currentTitle)">
          {{ titleText }}
        </span>
      </template>
      <template v-else>
        <input
          ref="titleInputRef"
          :value="editingTitle"
          @input="emit('update:editingTitle', ($event.target as HTMLInputElement).value)"
          type="text"
          class="w-full max-w-[200px] outline-none px-2 py-0.5 rounded text-base font-medium leading-relaxed text-center bg-white dark:bg-neutral-800 border border-transparent focus:border-[var(--theme-border)] shadow-sm"
          :style="{ color: textColor }"
          @blur="emit('saveTitle')"
          @keydown.enter="emit('saveTitle')"
          @keydown.escape="emit('cancelEditingTitle')"
        />
      </template>
    </div>

    <!-- Right buttons -->
    <div class="w-[60px] flex items-center justify-end gap-1">
      <!-- Plus button (list view only) -->
      <button v-if="viewMode === 'list' && !isAdding"
        @click="emit('startAdding')"
        class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
        :style="{ color: textColor }">
        <Plus class="w-4 h-4" />
      </button>

      <!-- Close button -->
      <button @click="emit('close')"
        class="close-btn shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
        :style="{ color: textColor }">
        <X class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>
