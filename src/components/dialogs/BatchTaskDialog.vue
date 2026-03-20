<script setup lang="ts">
import { ref, watch, computed, nextTick, onUnmounted } from 'vue';
import dayjs from 'dayjs';
import type { BatchTaskConfig } from '../../types';
import MiniCalendar from '../calendar/MiniCalendar.vue';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'confirm', config: BatchTaskConfig): void;
  (e: 'cancel'): void;
}>();

const startDate = ref(dayjs().format('YYYY-MM-DD'));
const endDate = ref(dayjs().add(1, 'month').format('YYYY-MM-DD'));
const cycleType = ref<'day' | 'week' | 'month'>('week');
const cycleCount = ref(1);
const title = ref('');
const description = ref('');

const inputRef = ref<HTMLInputElement | null>(null);

// 日历相关
const showStartCalendar = ref(false);
const showEndCalendar = ref(false);
const startCalendarDate = ref(dayjs());
const endCalendarDate = ref(dayjs());

// 计算预览信息
const previewInfo = computed(() => {
  const start = dayjs(startDate.value);
  const end = dayjs(endDate.value);
  const dates: string[] = [];

  let current = start;
  while (current.isBefore(end) || current.isSame(end, 'day')) {
    dates.push(current.format('YYYY-MM-DD'));
    switch (cycleType.value) {
      case 'day':
        current = current.add(cycleCount.value, 'day');
        break;
      case 'week':
        current = current.add(cycleCount.value, 'week');
        break;
      case 'month':
        current = current.add(cycleCount.value, 'month');
        break;
    }
  }

  return {
    count: dates.length,
    sampleDates: dates.slice(0, 3).map(d => dayjs(d).format('M月D日')).join('、') + (dates.length > 3 ? '...' : '')
  };
});

watch(() => props.visible, (newVal) => {
  if (newVal) {
    // 重置表单
    startDate.value = dayjs().format('YYYY-MM-DD');
    endDate.value = dayjs().add(1, 'month').format('YYYY-MM-DD');
    cycleType.value = 'week';
    cycleCount.value = 1;
    title.value = '';
    description.value = '';

    nextTick(() => inputRef.value?.focus());
    document.body.style.overflow = 'hidden';
  } else {
    document.body.style.overflow = '';
  }
});

const handleConfirm = () => {
  if (!title.value.trim()) {
    return;
  }

  emit('confirm', {
    startDate: startDate.value,
    endDate: endDate.value,
    cycleType: cycleType.value,
    cycleCount: cycleCount.value,
    title: title.value.trim(),
    description: description.value.trim() || undefined,
  });
};

const handleCancel = () => emit('cancel');

// 日历操作
function openStartCalendar() {
  startCalendarDate.value = dayjs(startDate.value);
  showStartCalendar.value = true;
}

function openEndCalendar() {
  endCalendarDate.value = dayjs(endDate.value);
  showEndCalendar.value = true;
}

function handleStartSelect(date: dayjs.Dayjs) {
  startDate.value = date.format('YYYY-MM-DD');
  showStartCalendar.value = false;
}

function handleEndSelect(date: dayjs.Dayjs) {
  endDate.value = date.format('YYYY-MM-DD');
  showEndCalendar.value = false;
}

function formatDateDisplay(dateStr: string): string {
  if (!dateStr) return '';
  return dayjs(dateStr).format('M月D日');
}

const handleKeydown = (e: KeyboardEvent) => {
  const isCmdOrCtrl = e.ctrlKey || e.metaKey;

  if (isCmdOrCtrl && (e.key === 'Enter' || e.key === 's' || e.key === 'S')) {
    e.preventDefault();
    handleConfirm();
    return;
  }

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
        <div class="bg-white/75 dark:bg-black/50 backdrop-blur-2xl w-full max-w-[400px] rounded-2xl shadow-[0_32px_64px_-12px_rgba(0,0,0,0.15)] border border-white/40 dark:border-white/10 overflow-hidden flex flex-col">

          <div class="px-5 pt-5 pb-3">
            <h3 class="text-[15px] font-bold text-gray-900 dark:text-white tracking-tight flex items-center gap-2">
              <span class="w-1.5 h-1.5 bg-blue-500 rounded-full animate-pulse"></span>
              批量添加任务
            </h3>
          </div>

          <div class="px-5 flex-grow space-y-4 pb-4">
            <!-- 任务标题 -->
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">任务标题</label>
              <input
                ref="inputRef"
                v-model="title"
                type="text"
                class="w-full px-3 py-2 text-sm bg-white/60 dark:bg-black/30 border border-gray-200 dark:border-gray-700 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none text-gray-900 dark:text-white placeholder-gray-400"
                placeholder="例如：每周一开会"
                @keydown="handleKeydown"
              />
            </div>

            <!-- 任务描述 -->
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">描述（可选）</label>
              <textarea
                v-model="description"
                class="w-full px-3 py-2 text-sm bg-white/60 dark:bg-black/30 border border-gray-200 dark:border-gray-700 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none text-gray-900 dark:text-white placeholder-gray-400 resize-none"
                rows="2"
                placeholder="任务的详细描述..."
                @keydown="handleKeydown"
              ></textarea>
            </div>

            <!-- 日期范围 -->
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">起始日期</label>
                <button
                  type="button"
                  @click="openStartCalendar"
                  class="w-full px-3 py-2 text-sm bg-white/60 dark:bg-black/30 border border-gray-200 dark:border-gray-700 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none text-left text-gray-900 dark:text-white"
                >
                  {{ formatDateDisplay(startDate) || '选择日期' }}
                </button>
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">结束日期</label>
                <button
                  type="button"
                  @click="openEndCalendar"
                  class="w-full px-3 py-2 text-sm bg-white/60 dark:bg-black/30 border border-gray-200 dark:border-gray-700 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none text-left text-gray-900 dark:text-white"
                >
                  {{ formatDateDisplay(endDate) || '选择日期' }}
                </button>
              </div>
            </div>

            <!-- MiniCalendar 弹窗 -->
            <MiniCalendar
              v-if="showStartCalendar"
              v-model:current-date="startCalendarDate"
              :visible="showStartCalendar"
              centered
              @select="handleStartSelect"
              @close="showStartCalendar = false"
            />
            <MiniCalendar
              v-if="showEndCalendar"
              v-model:current-date="endCalendarDate"
              :visible="showEndCalendar"
              centered
              @select="handleEndSelect"
              @close="showEndCalendar = false"
            />

            <!-- 循环设置 -->
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">循环方式</label>
              <div class="flex gap-2">
                <select
                  v-model="cycleType"
                  class="flex-1 px-3 py-2 text-sm bg-white/60 dark:bg-black/30 border border-gray-200 dark:border-gray-700 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none text-gray-900 dark:text-white"
                >
                  <option value="day">按天</option>
                  <option value="week">按周</option>
                  <option value="month">按月</option>
                </select>
                <input
                  v-model.number="cycleCount"
                  type="number"
                  min="1"
                  max="365"
                  class="w-20 px-3 py-2 text-sm bg-white/60 dark:bg-black/30 border border-gray-200 dark:border-gray-700 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none text-gray-900 dark:text-white"
                  placeholder="数量"
                />
                <span class="flex items-center text-sm text-gray-600 dark:text-gray-400">
                  {{ cycleType === 'day' ? '天' : cycleType === 'week' ? '周' : '月' }}
                </span>
              </div>
            </div>

            <!-- 预览信息 -->
            <div class="px-3 py-2 bg-blue-50/50 dark:bg-blue-900/20 rounded-lg">
              <p class="text-xs text-gray-600 dark:text-gray-400">
                将创建 <span class="font-semibold text-blue-600 dark:text-blue-400">{{ previewInfo.count }}</span> 个任务
              </p>
              <p v-if="previewInfo.count > 0" class="text-xs text-gray-500 dark:text-gray-500 mt-1">
                {{ previewInfo.sampleDates }}
              </p>
            </div>
          </div>

          <div class="px-5 pb-4 flex justify-between items-center">
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
              @click="handleConfirm"
              :disabled="!title.trim()"
              class="w-10 h-10 flex items-center justify-center rounded-xl bg-blue-500/10 hover:bg-blue-500 text-blue-600 hover:text-white transition-all active:scale-90 shadow-sm disabled:opacity-30 disabled:cursor-not-allowed"
              title="确认 (Ctrl + S)"
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

.pop-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.pop-leave-active { transition: all 0.25s cubic-bezier(0.4, 0, 1, 1); }
.pop-enter-from { opacity: 0; transform: scale(0.92) translateY(12px); }
.pop-leave-to { opacity: 0; transform: scale(0.95) translateY(4px); }

textarea::-webkit-scrollbar { width: 0; }
</style>
