<script setup lang="ts">
import { ref, watch, nextTick, onUnmounted, computed, onMounted } from 'vue';
import type { Schedule } from '../../types';
import { useSettings } from '../../composables/useSettings';
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
  const widthPercent = s.desc_dialog_width ?? 40;
  const heightPercent = s.desc_dialog_height ?? 70;
  return {
    '--theme-cell': 'var(--solid-bg)',
    '--theme-text': s.text_color,
    '--theme-text-muted': s.muted_text_color,
    '--theme-primary': s.primary_color,
    '--theme-border': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.05)'),
    '--dialog-border-width': `${s.cell_border_width || 1}px`,
    '--dialog-border-color': s.cell_border_color || (s.theme_mode === 'dark' ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.1)'),
    '--dialog-width': `${widthPercent}vw`,
    '--dialog-height': `${heightPercent}vh`,
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
        <div class="dialog-content w-full rounded-2xl shadow-lg flex flex-col overflow-hidden"
             :style="themeStyle">

          <div class="dialog-header px-4 pt-4 pb-2 shrink-0">
            <input
              v-model="content"
              type="text"
              class="title-input w-full text-base font-semibold tracking-tight bg-transparent border-none focus:outline-none focus:ring-0"
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
  backdrop-filter: blur(var(--backdrop-blur)) saturate(var(--backdrop-saturate));
  -webkit-backdrop-filter: blur(var(--backdrop-blur)) saturate(var(--backdrop-saturate));
  border: var(--dialog-border-width, 1px) solid var(--dialog-border-color);
  box-shadow: var(--shadow);
  max-width: var(--dialog-width, 40vw);
  height: var(--dialog-height, 70vh);
  max-height: 90vh;
}

/* 使用主题变量 */
.dialog-content {
  background-color: var(--theme-cell);
  color: var(--theme-text);
}

.dialog-content .title-input {
  color: var(--theme-text);
}

.dialog-content .title-input::placeholder {
  color: var(--theme-text-muted);
}

.dialog-header {
  border-bottom: 1px solid var(--border-light);
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.pop-enter-active { transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1); }
.pop-leave-active { transition: all 0.25s cubic-bezier(0.4, 0, 1, 1); }
.pop-enter-from { opacity: 0; transform: scale(0.92) translateY(12px); }
.pop-leave-to { opacity: 0; transform: scale(0.95) translateY(4px); }
</style>
