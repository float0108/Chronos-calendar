<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { X } from 'lucide-vue-next';

const props = withDefaults(defineProps<{
  themeStyle?: Record<string, string>;
  showCloseButton?: boolean;
  hideCloseButton?: boolean;
  showSearch?: boolean;
  searchPlaceholder?: string;
  modelValue?: string;
  title?: string;
}>(), {
  showCloseButton: true,
  hideCloseButton: false,
  showSearch: false,
  searchPlaceholder: 'Search...',
  modelValue: '',
  title: '',
});

const emit = defineEmits<{
  close: [];
  startDrag: [];
  'update:modelValue': [value: string];
  search: [keyword: string];
}>();

const textColor = computed(() => props.themeStyle?.['--theme-text'] || 'inherit');
const searchInputRef = ref<HTMLInputElement | null>(null);
const isSearchFocused = ref(false);

const displayValue = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val),
});

// 监听搜索输入
watch(displayValue, (val) => {
  emit('search', val);
});

// 当 showSearch 变为 true 时自动聚焦
watch(() => props.showSearch, (newVal) => {
  if (newVal) {
    nextTick(() => {
      searchInputRef.value?.focus();
    });
  }
});

function handleSearchInput(e: Event) {
  const target = e.target as HTMLInputElement;
  displayValue.value = target.value;
}

function handleSearchFocus() {
  isSearchFocused.value = true;
}

function handleSearchBlur() {
  isSearchFocused.value = false;
}
</script>

<template>
  <div class="window-title-bar flex items-center gap-2 px-3 py-2.5 shrink-0 select-none group">
    <slot name="left" />

    <div
      class="flex-1 min-w-0 flex justify-center items-center relative h-6"
      @mousedown="(e) => e.target === e.currentTarget && emit('startDrag')"
    >
      <!-- Search mode (when showSearch is true) -->
      <template v-if="showSearch">
        <span
          v-show="!isSearchFocused && !displayValue"
          class="text-base font-medium leading-relaxed transition-opacity cursor-text"
          :style="{ color: themeStyle?.['--theme-text'] || 'inherit' }"
          @mousedown.prevent="isSearchFocused = true; nextTick(() => searchInputRef?.focus())"
        >
          {{ title || 'Search' }}
        </span>
        <input
          ref="searchInputRef"
          v-show="isSearchFocused || displayValue"
          v-model="displayValue"
          type="text"
          :placeholder="title || searchPlaceholder"
          class="absolute inset-0 w-full h-full bg-black/5 dark:bg-white/5 rounded-md px-2 outline-none text-sm leading-relaxed text-center selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
          :style="{ color: themeStyle?.['--theme-text'] || 'inherit' }"
          @input="handleSearchInput"
          @focus="handleSearchFocus"
          @blur="handleSearchBlur"
          @mousedown.stop
        />
      </template>

      <!-- Default slot / center slot -->
      <template v-else>
        <slot name="center" />
        <slot />
      </template>
    </div>

    <slot name="right" />

    <button
      v-if="showCloseButton && !hideCloseButton"
      @click="emit('close')"
      class="close-btn shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all opacity-0 group-hover:opacity-100 hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
      :style="{ color: textColor }"
    >
      <X class="w-4 h-4" />
    </button>
  </div>
</template>
