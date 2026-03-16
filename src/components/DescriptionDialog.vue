<script setup lang="ts">
import { ref, watch, nextTick, onUnmounted, computed } from 'vue';
import type { Schedule, ViewMode } from '../types';

const props = defineProps<{
  visible: boolean;
  schedule: Schedule | null;
  viewMode: ViewMode;
}>();

const emit = defineEmits<{
  (e: 'save', scheduleId: number, description: string, dateField: 'create_date' | 'done_date' | null, dateValue: string | null): void;
  (e: 'cancel'): void;
}>();

const description = ref('');
const dateValue = ref('');
const textareaRef = ref<HTMLTextAreaElement | null>(null);

// 根据视图模式决定日期字段的标签和类型
const dateLabel = computed(() => {
  return props.viewMode === 'todo' ? '完成日期' : '创建日期';
});

const dateField = computed((): 'create_date' | 'done_date' | null => {
  if (!dateValue.value) return null;
  return props.viewMode === 'todo' ? 'done_date' : 'create_date';
});

watch(() => props.visible, (newVal) => {
  if (newVal) {
    description.value = props.schedule?.description || '';
    // 根据视图模式初始化日期值
    if (props.viewMode === 'todo') {
      dateValue.value = props.schedule?.done_date || '';
    } else {
      dateValue.value = props.schedule?.create_date || '';
    }
    nextTick(() => textareaRef.value?.focus());
    document.body.style.overflow = 'hidden';
  } else {
    document.body.style.overflow = '';
  }
});

const handleSave = () => {
  if (props.schedule) {
    emit('save', props.schedule.id!, description.value, dateField.value, dateValue.value || null);
  }
};

const handleCancel = () => emit('cancel');

const handleKeydown = (e: KeyboardEvent) => {
  const isCmdOrCtrl = e.ctrlKey || e.metaKey;

  // 1. Ctrl/Cmd + Enter 或 Ctrl/Cmd + S 保存
  if (isCmdOrCtrl && (e.key === 'Enter' || e.key === 's' || e.key === 'S')) {
    e.preventDefault(); // 阻止浏览器默认的保存行为 (如 Ctrl+S 弹出网页保存)
    handleSave();
    return;
  }

  // 2. Escape 取消
  if (e.key === 'Escape') {
    handleCancel();
  }
};

onUnmounted(() => { document.body.style.overflow = ''; });
</script>

<template>
  <Transition name="fade">
    <div
      v-if="visible"
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/10"
      @click.self="handleCancel"
    >
      <Transition name="pop">
        <div class="bg-white/75 dark:bg-black/50 backdrop-blur-2xl w-full max-w-[320px] rounded-2xl shadow-[0_32px_64px_-12px_rgba(0,0,0,0.15)] border border-white/40 dark:border-white/10 overflow-hidden flex flex-col">

          <div class="px-5 pt-5 pb-2">
            <h3 class="text-[14px] font-bold text-gray-900 dark:text-white tracking-tight flex items-center gap-2">
              <span class="w-1.5 h-1.5 bg-blue-500 rounded-full animate-pulse"></span>
              {{ schedule?.content || '详细备注' }}
            </h3>
          </div>

          <!-- 日期选择器 -->
          <div class="px-5 pb-2">
            <label class="flex items-center gap-2 text-[13px] text-gray-600 dark:text-gray-300">
              <span>{{ dateLabel }}</span>
              <input
                type="date"
                v-model="dateValue"
                class="flex-1 px-2.5 py-1.5 rounded-lg text-[13px] bg-white/50 dark:bg-white/10 border border-gray-200/60 dark:border-white/10 focus:outline-none focus:ring-2 focus:ring-blue-500/30 text-gray-800 dark:text-gray-100"
              />
            </label>
          </div>

          <div class="px-5 flex-grow">
            <textarea
              ref="textareaRef"
              v-model="description"
              class="w-full min-h-[200px] py-3 text-[15px] bg-transparent border-none focus:ring-0 outline-none resize-none text-gray-800 dark:text-gray-100 placeholder:text-gray-400/60 leading-relaxed font-medium"
              placeholder="输入描述内容... (Ctrl + S 保存)"
              @keydown="handleKeydown"
            ></textarea>
          </div>

          <div class="px-5 pb-4 flex justify-between items-center mt-2">
            <button
              @click="handleCancel"
              class="w-10 h-10 flex items-center justify-center rounded-xl hover:bg-black/[0.05] dark:hover:bg-white/[0.05] transition-all text-gray-400 hover:text-gray-600"
              title="取消 (Esc)"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>

            <button
              @click="handleSave"
              class="w-10 h-10 flex items-center justify-center rounded-xl bg-blue-500/10 hover:bg-blue-500 text-blue-600 hover:text-white transition-all active:scale-90 shadow-sm"
              title="保存 (Ctrl + S)"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
              </svg>
            </button>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

/* 更有质感的弹出动效 */
.pop-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.pop-leave-active { transition: all 0.25s cubic-bezier(0.4, 0, 1, 1); }
.pop-enter-from { opacity: 0; transform: scale(0.92) translateY(12px); }
.pop-leave-to { opacity: 0; transform: scale(0.95) translateY(4px); }

/* 隐藏滚动条但保留功能 */
textarea::-webkit-scrollbar { width: 0; }
</style>