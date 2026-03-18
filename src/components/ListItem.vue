<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted, computed, watch } from 'vue';
import { MinusCircle, Check, PlusCircle } from 'lucide-vue-next';
import MiniCalendar from './calendar/MiniCalendar.vue';
import dayjs from 'dayjs';
import { useSettings } from '../composables/useSettings';

const props = withDefaults(defineProps<{
  title?: string;
  preview?: string;
  date?: string;
  isDone?: boolean;
  editable?: boolean;
  centerCalendar?: boolean;
  isAddMode?: boolean;
}>(), {
  title: '',
  editable: true,
  centerCalendar: false,
  isAddMode: false,
});

const emit = defineEmits<{
  (e: 'update:title', value: string): void;
  (e: 'update:date', value: string): void;
  (e: 'delete'): void;
  (e: 'toggleDone'): void;
  (e: 'click'): void;
  (e: 'add', value: string): void;
  (e: 'cancel'): void;
}>();

const { currentMode } = useSettings();

const isEditing = ref(false);
const editTitle = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

let clickTimer: ReturnType<typeof setTimeout> | null = null;

// 日历相关
const showCalendar = ref(false);
const calendarPosition = ref<{ top: number; left: number } | undefined>(undefined);
const currentDate = ref(dayjs());

const themeMode = computed(() => currentMode.value);

const overlayColor = computed(() => {
  return themeMode.value === 'dark' ? 'rgba(0, 0, 0, 0.6)' : 'rgba(0, 0, 0, 0.4)';
});

function formatDate(dateStr?: string): string {
  return dateStr ? dayjs(dateStr).format('MM-DD') : '';
}

function clearClickTimer() {
  if (clickTimer) {
    clearTimeout(clickTimer);
    clickTimer = null;
  }
}

function startEdit() {
  if (!props.editable) return;
  editTitle.value = props.isAddMode ? '' : props.title;
  isEditing.value = true;
  nextTick(() => {
    inputRef.value?.focus();
    inputRef.value?.select();
  });
}

// isAddMode 时自动进入编辑状态
watch(() => props.isAddMode, (isAdd) => {
  if (isAdd) {
    nextTick(() => startEdit());
  }
}, { immediate: true });

function handleTitleClick() {
  if (isEditing.value) return;

  if (props.isAddMode) {
    startEdit();
    return;
  }

  clearClickTimer();
  clickTimer = setTimeout(() => {
    emit('click');
    clickTimer = null;
  }, 250);
}

function handleTitleDblclick() {
  clearClickTimer();
  if (!props.isAddMode) {
    startEdit();
  }
}

function handleSave() {
  const trimmed = editTitle.value.trim();

  if (props.isAddMode) {
    if (trimmed) {
      emit('add', trimmed);
      editTitle.value = '';
    } else {
      // 空内容时取消
      emit('cancel');
    }
  } else {
    if (trimmed && trimmed !== props.title) {
      emit('update:title', trimmed);
    } else {
      editTitle.value = props.title;
    }
  }
  isEditing.value = false;
}

function handleCancel() {
  isEditing.value = false;
  editTitle.value = '';
  if (props.isAddMode) {
    emit('cancel');
  }
}

function handleBlur() {
  if (isEditing.value) {
    // 新增模式下，blur 时如果内容为空则取消
    if (props.isAddMode && !editTitle.value.trim()) {
      handleCancel();
    } else {
      handleSave();
    }
  }
}

function handleContentClick() {
  if (!isEditing.value) {
    if (props.isAddMode) {
      startEdit();
    } else {
      emit('click');
    }
  }
}

function handleDateClick(e: MouseEvent) {
  if (!props.editable || props.isAddMode) return;

  currentDate.value = props.date ? dayjs(props.date) : dayjs();

  if (props.centerCalendar) {
    calendarPosition.value = undefined;
  } else {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    calendarPosition.value = {
      top: rect.bottom + 4,
      left: rect.right - 180
    };
  }
  showCalendar.value = true;
}

function handleDateSelect(date: dayjs.Dayjs) {
  emit('update:date', date.format('YYYY-MM-DD'));
  showCalendar.value = false;
}

function closeCalendar() {
  showCalendar.value = false;
}

function handleClickOutside() {
  if (showCalendar.value) showCalendar.value = false;
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  clearClickTimer();
});
</script>

<template>
  <div
    class="list-item group flex items-start gap-2 px-3 rounded-lg transition-colors duration-200"
    :class="[
      preview ? 'py-2' : 'py-1.5',
      isAddMode ? ' hover:bg-black/5 dark:hover:bg-white/5 opacity-80 hover:opacity-100 border-dashed' : ''
    ]"
    :style="{
      backgroundColor: 'var(--theme-cell)',
      border: isAddMode ? '1px dashed var(--theme-border)' : '1px solid var(--theme-border)',
    }"
  >
    <div class="flex-1 min-w-0" @click.stop="handleContentClick">
      <div class="flex items-center gap-2">
        <button
          v-if="isAddMode"
          class="relative shrink-0 w-4 h-4 flex items-center justify-center text-blue-500 dark:text-blue-400"
        >
          <PlusCircle class="w-4 h-4 opacity-70 group-hover:opacity-100 transition-opacity" />
        </button>
        <button
          v-else
          @click.stop="emit('delete')"
          class="relative shrink-0 w-4 h-4 flex items-center justify-center"
          :style="{ color: 'var(--theme-text-muted)' }"
        >
          <span class="w-1.5 h-1.5 rounded-full opacity-40 group-hover:opacity-0 transition-opacity"
            :style="{ backgroundColor: 'var(--theme-text-muted)' }"></span>
          <MinusCircle class="absolute inset-0 w-4 h-4 opacity-0 group-hover:opacity-100 hover:text-red-500 dark:hover:text-red-400 transition-opacity" />
        </button>

        <template v-if="!isEditing">
          <span
            v-if="isAddMode"
            class="text-[13px] font-medium leading-relaxed transition-colors flex-1 truncate"
            :style="{ color: 'var(--theme-text-muted)' }"
          >
            添加新项...
          </span>
          <span
            v-else
            class="text-[13px] font-medium leading-relaxed transition-colors flex-1 truncate "
            :style="{ color: isDone ? 'var(--theme-text-muted)' : 'var(--theme-text)' }"
            @click.stop="handleTitleClick"
            @dblclick.stop="handleTitleDblclick"
            @contextmenu.prevent.stop="emit('toggleDone')"
          >
            {{ title || '...' }}
          </span>
        </template>
        
        <template v-else>
          <div class="relative flex-1 min-w-0">
            <input
              ref="inputRef"
              v-model="editTitle"
              type="text"
              class="w-full outline-none pl-2 pr-8 py-0.5 -mx-1 rounded text-[13px] font-medium leading-relaxed selection:bg-[var(--theme-primary-alpha)] caret-[var(--theme-text)] bg-white dark:bg-neutral-800 border border-transparent focus:border-[var(--theme-border)] shadow-sm"
              :style="{ color: 'var(--theme-text)' }"
              :placeholder="isAddMode ? '输入内容并回车保存...' : ''"
              @blur="handleBlur"
              @keydown.enter="handleSave"
              @keydown.s.ctrl.prevent="handleSave"
              @keydown.escape="handleCancel"
              @click.stop
            />
            <button
              @mousedown.prevent="handleSave" 
              class="absolute right-1.5 top-1/2 -translate-y-1/2 w-6 h-6 flex items-center justify-center rounded-md text-emerald-500 hover:bg-emerald-50 dark:hover:bg-emerald-500/10 transition-colors"
              title="保存"
            >
              <Check class="w-4 h-4" stroke-width="2.5" />
            </button>
          </div>
        </template>

        <span
          v-if="date && !isAddMode"
          class="shrink-0 text-[11px] opacity-50 hover:opacity-80 transition-opacity cursor-pointer"
          :style="{ color: 'var(--theme-text-muted)' }"
          @click.stop="handleDateClick"
        >
          {{ formatDate(date) }}
        </span>
      </div>

      <div
        v-if="preview && !isAddMode"
        class="text-[12px] leading-relaxed truncate opacity-50 mt-0.5 pl-6"
        :style="{ color: 'var(--theme-text-muted)' }"
      >
        {{ preview }}
      </div>
    </div>

    <Teleport to="body">
      <Transition name="fade">
        <div
          v-if="showCalendar && centerCalendar"
          class="fixed inset-0 z-[10000]"
          :style="{ backgroundColor: overlayColor }"
          @click.stop="closeCalendar"
          @mousedown.stop
          @contextmenu.prevent
        >
        </div>
      </Transition>
      <MiniCalendar
        v-if="showCalendar && centerCalendar"
        v-model:current-date="currentDate"
        :visible="showCalendar && centerCalendar"
        centered
        @select="handleDateSelect"
        @close="closeCalendar"
      />
    </Teleport>

    <MiniCalendar
      v-if="!centerCalendar && showCalendar"
      v-model:current-date="currentDate"
      :visible="showCalendar"
      :position="calendarPosition"
      @select="handleDateSelect"
      @close="closeCalendar"
    />
  </div>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>