<script setup lang="ts">
import { ref, watch, nextTick, onUnmounted, computed } from 'vue';
import type { Schedule, ViewMode } from '../../types';
import ScheduleEditor from '../ScheduleEditor.vue';

const props = defineProps<{
  visible: boolean;
  schedule: Schedule | null;
  viewMode: ViewMode;
}>();

const emit = defineEmits<{
  (e: 'save', scheduleId: number, content: string, description: string, dateField: 'create_date' | 'done_date' | null, dateValue: string | null, fatherTask: number | null, markDone: boolean): void;
  (e: 'cancel'): void;
}>();

const content = ref('');
const description = ref('');
const createDateValue = ref('');
const doneDateValue = ref('');
const fatherTaskId = ref<number | null>(null);
const titleInputRef = ref<HTMLInputElement | null>(null);

// 根据视图模式决定显示哪个日期
const showCreateDate = computed(() => props.viewMode === 'done');
const showDoneDate = computed(() => props.viewMode === 'todo');

// 计算实际的日期字段
const dateField = computed((): 'create_date' | 'done_date' | null => {
  if (props.viewMode === 'todo' && doneDateValue.value) return 'done_date';
  if (props.viewMode === 'done' && createDateValue.value) return 'create_date';
  return null;
});

// 获取日期值
const dateValue = computed(() => {
  return props.viewMode === 'todo' ? doneDateValue.value : createDateValue.value;
});

watch(() => props.visible, (newVal) => {
  if (newVal) {
    content.value = props.schedule?.content || '';
    description.value = props.schedule?.description || '';
    fatherTaskId.value = props.schedule?.father_task || null;

    // 日期初始化逻辑：
    // - 如果 is_done=false，日期栏留空
    // - 如果 is_done=true，显示 done_date 或 create_date
    if (props.schedule?.is_done) {
      if (props.viewMode === 'todo') {
        doneDateValue.value = props.schedule?.done_date || '';
        createDateValue.value = '';
      } else {
        createDateValue.value = props.schedule?.create_date || '';
        doneDateValue.value = '';
      }
    } else {
      createDateValue.value = '';
      doneDateValue.value = '';
    }

    nextTick(() => titleInputRef.value?.focus());
    document.body.style.overflow = 'hidden';
  } else {
    document.body.style.overflow = '';
  }
});

const handleSave = () => {
  if (props.schedule) {
    // 如果日期有值且原来未完成，需要标记为完成
    const markDone = !!dateValue.value && !props.schedule.is_done;
    emit('save', props.schedule.id!, content.value.trim() || props.schedule.content, description.value, dateField.value, dateValue.value || null, fatherTaskId.value, markDone);
  }
};

const handleCancel = () => emit('cancel');

const handleKeydown = (e: KeyboardEvent) => {
  const isCmdOrCtrl = e.ctrlKey || e.metaKey;

  // Ctrl/Cmd + Enter 或 Ctrl/Cmd + S 保存
  if (isCmdOrCtrl && (e.key === 'Enter' || e.key === 's' || e.key === 'S')) {
    e.preventDefault();
    handleSave();
    return;
  }

  // Escape 取消
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
            <input
              ref="titleInputRef"
              v-model="content"
              type="text"
              class="w-full text-[14px] font-bold text-gray-900 dark:text-white tracking-tight bg-transparent border-none focus:outline-none focus:ring-0 placeholder:text-gray-400"
              placeholder="输入标题..."
              @keydown="handleKeydown"
            />
          </div>

          <div class="px-5 flex-grow">
            <ScheduleEditor
              v-model:description="description"
              v-model:create-date="createDateValue"
              v-model:done-date="doneDateValue"
              v-model:father-task-id="fatherTaskId"
              :show-content="false"
              :show-create-date="showCreateDate"
              :show-done-date="showDoneDate"
              :show-father-task="true"
              :editable-father-task="true"
            />
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
</style>
