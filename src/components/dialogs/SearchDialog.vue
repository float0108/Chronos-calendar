<script setup lang="ts">
import { ref, watch, nextTick, onUnmounted, onMounted, computed } from 'vue';
import { X, Search, XCircle } from 'lucide-vue-next';
import ListItem from '../ListItem.vue';
import { searchSchedules } from '../../api/database';
import { useSettings } from '../../composables/useSettings';
import type { Schedule } from '../../types';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'navigate', payload: { date: string; viewMode: 'todo' | 'done' }): void;
}>();

const { currentSettings } = useSettings();

// 为组件提供主题变量
const themeStyle = computed(() => {
  const s = currentSettings.value;
  if (!s) return {};
  const widthPercent = s.desc_dialog_width ?? 40;
  const heightPercent = s.desc_dialog_height ?? 70;
  return {
    '--theme-cell': 'var(--solid-bg)',
    '--theme-text': s.text_color,
    '--theme-text-muted': s.muted_text_color,
    '--theme-primary': s.primary_color,
    '--theme-primary-alpha': 'rgba(0,0,0,0.05)',
    '--theme-border': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.05)'),
    '--dialog-border-width': `${s.cell_border_width || 1}px`,
    '--dialog-border-color': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.1)'),
    '--dialog-width': `${widthPercent}vw`,
    '--dialog-height': `${heightPercent}vh`,
  };
});

const searchKeyword = ref('');
const searchResults = ref<Schedule[]>([]);
const isLoading = ref(false);

// 防抖搜索
let searchTimeout: ReturnType<typeof setTimeout> | null = null;

async function performSearch() {
  if (!searchKeyword.value.trim()) {
    searchResults.value = [];
    return;
  }

  isLoading.value = true;
  try {
    searchResults.value = await searchSchedules(searchKeyword.value);
  } catch (error) {
    console.error('Search failed:', error);
    searchResults.value = [];
  } finally {
    isLoading.value = false;
  }
}

watch(searchKeyword, () => {
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
  searchTimeout = setTimeout(() => {
    performSearch();
  }, 200);
});

watch(() => props.visible, (newVal) => {
  if (newVal) {
    searchKeyword.value = '';
    searchResults.value = [];
    nextTick(() => searchInputRef.value?.focus());
    document.body.style.overflow = 'hidden';
  } else {
    document.body.style.overflow = '';
  }
});

function handleResultClick(schedule: Schedule) {
  const isDone = schedule.is_done;
  const targetDate = isDone && schedule.done_date ? schedule.done_date : schedule.create_date;
  const viewMode = isDone && schedule.done_date ? 'done' : 'todo';

  emit('navigate', { date: targetDate, viewMode });
}

function handleClose() {
  emit('close');
}

const handleKeydown = (e: KeyboardEvent) => {
  if (!props.visible) return;

  if (e.key === 'Escape') {
    e.preventDefault();
    handleClose();
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
  document.body.style.overflow = '';
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
});

// DOM Refs
const searchInputRef = ref<HTMLInputElement | null>(null);
</script>

<template>
  <Transition name="fade">
    <div
      v-if="visible"
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/10"
      @click.self="handleClose"
    >
      <Transition name="pop">
        <div class="dialog-content w-full rounded-2xl shadow-lg flex flex-col overflow-hidden"
             :style="themeStyle">

          <div class="dialog-header px-4 pt-4 pb-2 shrink-0">
            <div class="flex items-center gap-2">
              <Search class="w-4 h-4 opacity-60 shrink-0" />
              <input
                ref="searchInputRef"
                v-model="searchKeyword"
                type="text"
                placeholder="搜索日程..."
                class="flex-1 h-7 bg-black/5 dark:bg-white/5 rounded-md pl-3 pr-8 outline-none text-sm leading-relaxed selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)]"
              />
              <button
                v-if="searchKeyword"
                @click="searchKeyword = ''"
                class="shrink-0 w-5 h-5 flex items-center justify-center rounded transition-colors hover:bg-black/10 dark:hover:bg-white/10"
              >
                <XCircle class="w-3.5 h-3.5 opacity-60" />
              </button>
              <button
                @click="handleClose"
                class="shrink-0 w-6 h-6 flex items-center justify-center rounded transition-all hover:bg-black/10 dark:hover:bg-white/10 active:scale-95"
              >
                <X class="w-4 h-4 opacity-60" />
              </button>
            </div>
          </div>

          <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar px-4 py-3">
            <!-- 加载状态 -->
            <div v-if="isLoading" class="flex items-center justify-center py-8">
              <div class="animate-spin w-5 h-5 border-2 border-[var(--theme-primary)] border-t-transparent rounded-full"></div>
            </div>

            <!-- 搜索结果列表 -->
            <div v-else-if="searchResults.length > 0" class="space-y-2 list-none">
              <ListItem
                v-for="schedule in searchResults"
                :key="schedule.id"
                :title="schedule.content"
                :preview="schedule.description"
                :date="schedule.create_date"
                :is-done="schedule.is_done"
                :editable="false"
                @click="handleResultClick(schedule)"
              />
            </div>

            <!-- 空状态 -->
            <div v-else-if="searchKeyword.trim()" class="flex flex-col items-center justify-center py-12 pointer-events-none">
              <div class="p-4 rounded-full opacity-10">
                <Search class="w-6 h-6" />
              </div>
              <div class="mt-3 text-sm opacity-50">
                未找到相关日程
              </div>
            </div>

            <!-- 初始状态 -->
            <div v-else class="flex flex-col items-center justify-center py-12 pointer-events-none">
              <div class="p-4 rounded-full opacity-10">
                <Search class="w-6 h-6" />
              </div>
              <div class="mt-3 text-sm opacity-50">
                输入关键词搜索日程
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped>
.dialog-content {
  backdrop-filter: blur(var(--backdrop-blur)) saturate(var(--backdrop-saturate));
  -webkit-backdrop-filter: blur(var(--backdrop-blur)) saturate(var(--backdrop-saturate));
  border: var(--dialog-border-width, 1px) solid var(--dialog-border-color);
  box-shadow: var(--shadow);
  max-width: var(--dialog-width, 40vw);
  height: var(--dialog-height, 70vh);
  max-height: 90vh;
  background-color: var(--theme-cell);
  color: var(--theme-text);
}

.dialog-header {
  border-bottom: 1px solid var(--border-light);
}

.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: transparent transparent;
  transition: scrollbar-color 0.3s ease;
}

.custom-scrollbar:hover {
  scrollbar-color: var(--theme-border) transparent;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: transparent;
  border-radius: 4px;
}

.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background-color: var(--theme-border);
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.pop-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.pop-leave-active { transition: all 0.25s cubic-bezier(0.4, 0, 1, 1); }
.pop-enter-from { opacity: 0; transform: scale(0.92) translateY(12px); }
.pop-leave-to { opacity: 0; transform: scale(0.95) translateY(4px); }
</style>
