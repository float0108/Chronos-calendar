<script setup lang="ts">
import { ref, watch, nextTick, onUnmounted, computed, onMounted } from 'vue';
import type { Schedule } from '../../types';
import { useSettings } from '../../composables/useSettings';
import { hexToRgba, adjustBrightness } from '../../utils/color';
import ScheduleEditor from '../ScheduleEditor.vue';

const props = defineProps<{
  visible: boolean;
  schedule: Schedule | null;
}>();

const emit = defineEmits<{
  (e: 'save', scheduleId: number, content: string, description: string, createDate: string, doneDate: string, fatherTask: number | null): void;
  (e: 'cancel'): void;
}>();

const { currentSettings } = useSettings();

const content = ref('');
const description = ref('');
const createDateValue = ref('');
const doneDateValue = ref('');
const fatherTaskId = ref<number | null>(null);
const scheduleEditorRef = ref<InstanceType<typeof ScheduleEditor> | null>(null);

// 为 ScheduleEditor 提供主题变量
const themeStyle = computed(() => {
  const s = currentSettings.value;
  if (!s) return {};
  const cellOpacity = s.cell_opacity / 100;
  return {
    '--theme-cell': hexToRgba(s.cell_color, cellOpacity),
    '--theme-text': s.text_color,
    '--theme-text-muted': adjustBrightness(s.text_color, 50),
    '--theme-primary': s.primary_color,
    '--theme-border': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.05)'),
  };
});

watch(() => props.visible, (newVal) => {
  if (newVal) {
    content.value = props.schedule?.content || '';
    description.value = props.schedule?.description || '';
    fatherTaskId.value = props.schedule?.father_task ?? null;
    createDateValue.value = props.schedule?.create_date || '';
    doneDateValue.value = props.schedule?.done_date || '';

    nextTick(() => scheduleEditorRef.value?.loadTasks());
    document.body.style.overflow = 'hidden';
  } else {
    document.body.style.overflow = '';
  }
});

const handleSave = () => {
  if (props.schedule) {
    emit('save',
      props.schedule.id!,
      content.value.trim() || props.schedule.content,
      description.value,
      createDateValue.value,
      doneDateValue.value,
      fatherTaskId.value
    );
  }
};

const handleCancel = () => emit('cancel');

const handleKeydown = (e: KeyboardEvent) => {
  if (!props.visible) return;

  const isCmdOrCtrl = e.ctrlKey || e.metaKey;

  // Ctrl/Cmd + Enter 或 Ctrl/Cmd + S 保存
  if (isCmdOrCtrl && (e.key === 'Enter' || e.key === 's' || e.key === 'S')) {
    e.preventDefault();
    handleSave();
  }
  // ESC 由 ScheduleEditor 处理
};

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
  document.body.style.overflow = '';
});
</script>

<template>
  <Transition name="fade">
    <div
      v-if="visible"
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/10"
      @click.self="handleCancel"
    >
      <Transition name="pop">
        <div class="dialog-content w-full max-w-[320px] rounded-2xl shadow-lg flex flex-col max-h-[85vh] overflow-hidden" :style="themeStyle">

          <div class="dialog-header px-4 pt-4 pb-2 shrink-0">
            <input
              v-model="content"
              type="text"
              class="dialog-input w-full text-[14px] font-semibold tracking-tight bg-transparent border-none focus:outline-none focus:ring-0"
              placeholder="输入标题..."
            />
          </div>

          <div class="flex-1 min-h-0 overflow-y-auto px-4 py-3">
            <ScheduleEditor
              ref="scheduleEditorRef"
              class="h-full"
              v-model:description="description"
              v-model:create-date="createDateValue"
              v-model:done-date="doneDateValue"
              v-model:father-task-id="fatherTaskId"
              :show-content="false"
              :show-father-task="true"
              :editable-father-task="true"
              :calendar-centered="true"
              @save="handleSave"
              @cancel="handleCancel"
            />
          </div>

        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped>
.dialog-content {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--backdrop-blur)) saturate(var(--backdrop-saturate));
  -webkit-backdrop-filter: blur(var(--backdrop-blur)) saturate(var(--backdrop-saturate));
  border: 1px solid var(--border-light);
  box-shadow: var(--shadow);
}

.dialog-header {
  border-bottom: 1px solid var(--border-light);
}

.dialog-input {
  color: var(--text-primary);
}

.dialog-input::placeholder {
  color: var(--text-muted);
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.pop-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.pop-leave-active { transition: all 0.25s cubic-bezier(0.4, 0, 1, 1); }
.pop-enter-from { opacity: 0; transform: scale(0.92) translateY(12px); }
.pop-leave-to { opacity: 0; transform: scale(0.95) translateY(4px); }
</style>
