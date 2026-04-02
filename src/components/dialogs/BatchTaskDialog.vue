<script setup lang="ts">
import { ref, watch, computed, nextTick, onUnmounted } from 'vue';
import { FileText, Calendar, Repeat } from 'lucide-vue-next';
import dayjs from 'dayjs';
import type { BatchTaskConfig } from '../../types';
import { useSettings } from '../../composables/useSettings';
import { hexToRgba } from '../../utils/color';
import MiniCalendar from '../calendar/MiniCalendar.vue';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'confirm', config: BatchTaskConfig): void;
  (e: 'cancel'): void;
}>();

const { currentSettings } = useSettings();

const startDate = ref(dayjs().format('YYYY-MM-DD'));
const endDate = ref(dayjs().add(1, 'month').format('YYYY-MM-DD'));
const cycleType = ref<'day' | 'week' | 'month'>('week');
const cycleCount = ref(1);
const title = ref('');
const description = ref('');

const inputRef = ref<HTMLInputElement | null>(null);

// 主题样式
const themeStyle = computed(() => {
  const s = currentSettings.value;
  if (!s) return {};
  const cellOpacity = s.cell_opacity / 100;
  return {
    '--theme-cell': hexToRgba(s.cell_color, cellOpacity),
    '--theme-primary': s.primary_color,
    '--theme-primary-bg': hexToRgba(s.primary_color, 0.08),
    '--theme-text': s.text_color,
    '--theme-text-muted': s.muted_text_color,
    '--theme-border': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.06)'),
    '--dialog-border-width': `${s.cell_border_width || 1}px`,
    '--dialog-border-color': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.1)'),
  };
});

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
        <div class="dialog-content w-full rounded-2xl shadow-lg overflow-hidden flex flex-col"
             :class="{ 'dark-mode': currentSettings?.theme_mode === 'dark', 'light-mode': currentSettings?.theme_mode !== 'dark' }"
             :style="themeStyle">

          <!-- 标题区域 -->
          <div class="dialog-header px-4 pt-4 pb-2 shrink-0">
            <input
              ref="inputRef"
              v-model="title"
              type="text"
              class="title-input w-full text-base font-semibold tracking-tight bg-transparent border-none focus:outline-none focus:ring-0"
              placeholder="输入任务标题..."
              @keydown="handleKeydown"
            />
          </div>

          <div class="flex-1 min-h-0 overflow-y-auto px-4 py-3 space-y-3 text-sm">
            <!-- 任务描述 -->
            <div class="flex flex-col gap-1">
              <label class="flex items-center gap-1.5 font-medium" :style="{ color: 'var(--theme-text-muted)' }">
                <FileText class="w-3 h-3" />
                <span>描述</span>
              </label>
              <textarea
                v-model="description"
                class="schedule-input w-full px-2 py-1.5 rounded outline-none resize-none transition-colors"
                :style="{
                  backgroundColor: 'var(--theme-cell)',
                  borderColor: 'var(--theme-border)',
                  color: 'var(--theme-text)',
                }"
                rows="2"
                placeholder="任务的详细描述..."
                @keydown="handleKeydown"
              ></textarea>
            </div>

            <!-- 日期范围 -->
            <div class="grid grid-cols-2 gap-2">
              <div class="flex flex-col gap-1">
                <label class="flex items-center gap-1.5 font-medium" :style="{ color: 'var(--theme-text-muted)' }">
                  <Calendar class="w-3 h-3" />
                  <span>起始日期</span>
                </label>
                <div class="date-picker-wrapper relative">
                  <button
                    type="button"
                    @click="openStartCalendar"
                    class="schedule-input w-full px-2 py-1.5 rounded text-left outline-none focus:ring-1 transition-colors"
                    :style="{
                      backgroundColor: 'var(--theme-cell)',
                      borderColor: 'var(--theme-border)',
                      color: 'var(--theme-text)',
                      '--tw-ring-color': 'var(--theme-primary)',
                    }"
                  >
                    {{ formatDateDisplay(startDate) || '选择日期' }}
                  </button>
                  <MiniCalendar
                    v-if="showStartCalendar"
                    v-model:current-date="startCalendarDate"
                    :visible="showStartCalendar"
                    centered
                    @select="handleStartSelect"
                    @close="showStartCalendar = false"
                  />
                </div>
              </div>
              <div class="flex flex-col gap-1">
                <label class="flex items-center gap-1.5 font-medium" :style="{ color: 'var(--theme-text-muted)' }">
                  <Calendar class="w-3 h-3" />
                  <span>结束日期</span>
                </label>
                <div class="date-picker-wrapper relative">
                  <button
                    type="button"
                    @click="openEndCalendar"
                    class="schedule-input w-full px-2 py-1.5 rounded text-left outline-none focus:ring-1 transition-colors"
                    :style="{
                      backgroundColor: 'var(--theme-cell)',
                      borderColor: 'var(--theme-border)',
                      color: 'var(--theme-text)',
                      '--tw-ring-color': 'var(--theme-primary)',
                    }"
                  >
                    {{ formatDateDisplay(endDate) || '选择日期' }}
                  </button>
                  <MiniCalendar
                    v-if="showEndCalendar"
                    v-model:current-date="endCalendarDate"
                    :visible="showEndCalendar"
                    centered
                    @select="handleEndSelect"
                    @close="showEndCalendar = false"
                  />
                </div>
              </div>
            </div>

            <!-- 循环设置 -->
            <div class="flex flex-col gap-1">
              <label class="flex items-center gap-1.5 font-medium" :style="{ color: 'var(--theme-text-muted)' }">
                <Repeat class="w-3 h-3" />
                <span>循环方式</span>
              </label>
              <div class="flex gap-2 items-center">
                <select
                  v-model="cycleType"
                  class="schedule-input flex-1 px-2 py-1.5 rounded outline-none focus:ring-1 transition-colors appearance-none cursor-pointer"
                  :style="{
                    backgroundColor: 'var(--theme-cell)',
                    borderColor: 'var(--theme-border)',
                    color: 'var(--theme-text)',
                    '--tw-ring-color': 'var(--theme-primary)',
                  }"
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
                  class="schedule-input w-14 px-2 py-1.5 rounded outline-none focus:ring-1 transition-colors text-center"
                  :style="{
                    backgroundColor: 'var(--theme-cell)',
                    borderColor: 'var(--theme-border)',
                    color: 'var(--theme-text)',
                    '--tw-ring-color': 'var(--theme-primary)',
                  }"
                  placeholder="数量"
                />
                <span class="opacity-50 whitespace-nowrap" :style="{ color: 'var(--theme-text-muted)' }">
                  {{ cycleType === 'day' ? '天/次' : cycleType === 'week' ? '周/次' : '月/次' }}
                </span>
              </div>
            </div>

            <!-- 预览信息 -->
            <div class="px-3 py-2 rounded-lg" :style="{ backgroundColor: 'var(--theme-primary-bg)' }">
              <p class="opacity-60">
                将创建 <span class="font-semibold" :style="{ color: 'var(--theme-primary)' }">{{ previewInfo.count }}</span> 个任务
              </p>
              <p v-if="previewInfo.count > 0" class="opacity-40 mt-1">
                {{ previewInfo.sampleDates }}
              </p>
            </div>
          </div>

          <!-- 底部按钮 -->
          <div class="dialog-footer px-4 py-3 flex justify-between items-center shrink-0">
            <button
              @click="handleCancel"
              class="action-btn w-10 h-10 flex items-center justify-center rounded-xl transition-all"
              title="取消 (Esc)"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>

            <button
              @click="handleConfirm"
              :disabled="!title.trim()"
              class="confirm-btn flex-1 h-10 flex items-center justify-center gap-1.5 rounded-xl text-white transition-opacity font-medium shadow-sm active:scale-[0.98]"
              title="确认 (Ctrl + S)"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
              </svg>
              <span>创建</span>
            </button>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped>
.dialog-content {
  max-width: 420px;
  backdrop-filter: blur(var(--backdrop-blur)) saturate(var(--backdrop-saturate));
  -webkit-backdrop-filter: blur(var(--backdrop-blur)) saturate(var(--backdrop-saturate));
  border: var(--dialog-border-width, 1px) solid var(--dialog-border-color);
  box-shadow: var(--shadow);
}

/* 亮色模式 */
.dialog-content.light-mode {
  background-color: #ffffff;
  color: #000000;
}

/* 暗色模式 */
.dialog-content.dark-mode {
  background-color: #000000;
  color: #ffffff;
}

.dialog-header {
  border-bottom: 1px solid var(--border-light);
}

/* 标题输入框 - 幽灵样式 */
.dialog-content.light-mode .title-input {
  color: #000000;
}

.dialog-content.dark-mode .title-input {
  color: #ffffff;
}

.dialog-content.light-mode .title-input::placeholder {
  color: rgba(0, 0, 0, 0.5);
}

.dialog-content.dark-mode .title-input::placeholder {
  color: rgba(255, 255, 255, 0.5);
}

/* 输入框样式 */
.schedule-input {
  border: 1px solid var(--theme-border);
}

/* 下拉框美化 */
.schedule-input[type="number"] {
  -moz-appearance: textfield;
}

.schedule-input[type="number"]::-webkit-outer-spin-button,
.schedule-input[type="number"]::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

/* select 下拉箭头 */
select.schedule-input {
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%236b7280' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  padding-right: 32px;
}

.dialog-content.dark-mode select.schedule-input {
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%239ca3af' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
}

/* 底部按钮区域 */
.dialog-footer {
  border-top: 1px solid var(--border-light);
}

/* 取消按钮 */
.action-btn {
  background-color: transparent;
  color: var(--text-muted);
}

.action-btn:hover {
  background-color: var(--theme-cell);
}

/* 确认按钮 */
.confirm-btn {
  background-color: var(--theme-primary);
}

.confirm-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

/* 下拉选择框样式修复 */
.dialog-content.dark-mode select option {
  background-color: #1a1a1a;
  color: #ffffff;
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.pop-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.pop-leave-active { transition: all 0.25s cubic-bezier(0.4, 0, 1, 1); }
.pop-enter-from { opacity: 0; transform: scale(0.92) translateY(12px); }
.pop-leave-to { opacity: 0; transform: scale(0.95) translateY(4px); }

textarea::-webkit-scrollbar { width: 0; }
</style>
