<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { Calendar, CheckCircle, Link2, FileText, X, Check, Plus } from 'lucide-vue-next';
import { loadMainTasks, type MainTask } from '../api/database';
import MiniCalendar from './calendar/MiniCalendar.vue';
import dayjs from 'dayjs';

const props = withDefaults(defineProps<{
  content?: string;
  description?: string;
  createDate?: string;
  doneDate?: string;
  fatherTaskId?: number | null;
  showContent?: boolean;
  showCreateDate?: boolean;
  showDoneDate?: boolean;
  showFatherTask?: boolean;
  editableFatherTask?: boolean;
  calendarCentered?: boolean;
}>(), {
  showCreateDate: true,
  showDoneDate: true,
  calendarCentered: false,
});

const emit = defineEmits<{
  (e: 'update:content', value: string): void;
  (e: 'update:description', value: string): void;
  (e: 'update:createDate', value: string): void;
  (e: 'update:doneDate', value: string): void;
  (e: 'update:fatherTaskId', value: number | null): void;
  (e: 'create-task', keyword: string): void;
  (e: 'save'): void;
  (e: 'cancel'): void;
}>();

// 本地状态
const localContent = ref(props.content || '');
const localDescription = ref(props.description || '');
const localCreateDate = ref(props.createDate || '');
const localDoneDate = ref(props.doneDate || '');
const localFatherTaskId = ref<number | null>(props.fatherTaskId ?? null);

// 主任务列表
const mainTasks = ref<MainTask[]>([]);
const searchKeyword = ref('');
const showTaskDropdown = ref(false);

// 日历相关
const showCreateDateCalendar = ref(false);
const showDoneDateCalendar = ref(false);
const createDateValue = ref(dayjs());
const doneDateValue = ref(dayjs());

// 过滤后的任务列表
const filteredTasks = computed(() => {
  const keyword = searchKeyword.value.trim().toLowerCase();
  let tasks = mainTasks.value;

  if (keyword) {
    tasks = tasks.filter(t =>
      t.content.toLowerCase().includes(keyword) ||
      (t.description && t.description.toLowerCase().includes(keyword))
    );
  }

  return tasks.sort((a, b) => {
    if (a.is_done !== b.is_done) return a.is_done ? 1 : -1;
    if (a.priority !== b.priority) return b.priority - a.priority;
    return a.id! - b.id!;
  });
});

// 获取当前选中的任务名称
const selectedTaskName = computed(() => {
  if (!localFatherTaskId.value) return '';
  const task = mainTasks.value.find(t => t.id === localFatherTaskId.value);
  return task ? task.content : '';
});

// 加载主任务列表
async function loadTasks() {
  mainTasks.value = await loadMainTasks();
}

// 监听 props 变化
watch(() => props.content, (val) => { localContent.value = val || ''; });
watch(() => props.description, (val) => { localDescription.value = val || ''; });
watch(() => props.createDate, (val) => { localCreateDate.value = val || ''; });
watch(() => props.doneDate, (val) => { localDoneDate.value = val || ''; });
watch(() => props.fatherTaskId, (val) => { localFatherTaskId.value = val ?? null; });

// 监听本地变化并 emit
watch(localContent, (val) => { emit('update:content', val); });
watch(localDescription, (val) => { emit('update:description', val); });
watch(localCreateDate, (val) => { emit('update:createDate', val); });
watch(localDoneDate, (val) => { emit('update:doneDate', val); });
watch(localFatherTaskId, (val) => { emit('update:fatherTaskId', val); });

function handleTaskInputFocus() {
  showTaskDropdown.value = true;
}

function handleTaskInputBlur() {
  setTimeout(() => {
    showTaskDropdown.value = false;
  }, 150);
}

function selectTask(task: MainTask | null) {
  localFatherTaskId.value = task?.id ?? null;
  searchKeyword.value = '';
  showTaskDropdown.value = false;
}

function clearTask() {
  localFatherTaskId.value = null;
  searchKeyword.value = '';
}

function handleCreateNewTask() {
  const keyword = searchKeyword.value.trim();
  if (keyword) {
    emit('create-task', keyword);
    searchKeyword.value = '';
    showTaskDropdown.value = false;
  }
}

function handleSave() { emit('save'); }
function handleCancel() { emit('cancel'); }

// 日历操作
function openCreateDateCalendar() {
  createDateValue.value = localCreateDate.value ? dayjs(localCreateDate.value) : dayjs();
  showCreateDateCalendar.value = true;
}

function openDoneDateCalendar() {
  doneDateValue.value = localDoneDate.value ? dayjs(localDoneDate.value) : dayjs();
  showDoneDateCalendar.value = true;
}

function handleCreateDateSelect(date: dayjs.Dayjs) {
  localCreateDate.value = date.format('YYYY-MM-DD');
  showCreateDateCalendar.value = false;
}

function handleDoneDateSelect(date: dayjs.Dayjs) {
  localDoneDate.value = date.format('YYYY-MM-DD');
  showDoneDateCalendar.value = false;
}

function closeAllCalendars() {
  showCreateDateCalendar.value = false;
  showDoneDateCalendar.value = false;
}

function formatDateDisplay(dateStr: string): string {
  return dateStr ? dayjs(dateStr).format('M月D日') : '';
}

function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (!target.closest('.date-picker-wrapper') && !target.closest('.mini-calendar')) {
    closeAllCalendars();
  }
}

function handleKeydown(e: KeyboardEvent) {
  // Escape 取消
  if (e.key === 'Escape') {
    e.preventDefault();
    handleCancel();
  }
}

onMounted(() => {
  if (props.showFatherTask) {
    loadTasks();
  }
  document.addEventListener('click', handleClickOutside);
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  document.removeEventListener('keydown', handleKeydown);
});

defineExpose({ loadTasks });
</script>

<template>
  <div class="schedule-editor flex flex-col gap-3 h-full text-sm">

    <div v-if="showContent" class="flex flex-col gap-1 shrink-0">
      <label class="flex items-center gap-1.5 font-medium" :style="{ color: 'var(--theme-text-muted)' }">
        <span>标题</span>
      </label>
      <input
        v-model="localContent"
        type="text"
        placeholder="输入标题..."
        class="schedule-input w-full px-2 py-1.5 rounded text-sm outline-none focus:ring-1 transition-colors"
        :style="{
          backgroundColor: 'var(--theme-cell)',
          borderColor: 'var(--theme-border)',
          color: 'var(--theme-text)',
          '--tw-ring-color': 'var(--theme-primary)',
        }"
      />
    </div>

    <div class="grid grid-cols-2 gap-2 shrink-0">
      <div class="flex flex-col gap-1">
        <label class="flex items-center gap-1.5 font-medium" :style="{ color: 'var(--theme-text-muted)' }">
          <Calendar class="w-3 h-3" />
          <span>创建日期</span>
        </label>
        <div class="date-picker-wrapper relative">
          <button
            type="button"
            @click="openCreateDateCalendar"
            class="schedule-input w-full px-2 py-1.5 rounded text-sm text-left outline-none focus:ring-1 transition-colors"
            :style="{
              backgroundColor: 'var(--theme-cell)',
              borderColor: 'var(--theme-border)',
              color: 'var(--theme-text)',
              '--tw-ring-color': 'var(--theme-primary)',
            }"
          >
            {{ formatDateDisplay(localCreateDate) || '选择日期' }}
          </button>
          <MiniCalendar
            v-model:current-date="createDateValue"
            :visible="showCreateDateCalendar"
            :centered="calendarCentered"
            @select="handleCreateDateSelect"
            @close="showCreateDateCalendar = false"
          />
        </div>
      </div>

      <div class="flex flex-col gap-1">
        <label class="flex items-center gap-1.5 font-medium" :style="{ color: 'var(--theme-text-muted)' }">
          <CheckCircle class="w-3 h-3" />
          <span>完成日期</span>
        </label>
        <div class="date-picker-wrapper relative">
          <button
            type="button"
            @click="openDoneDateCalendar"
            class="schedule-input w-full px-2 py-1.5 rounded text-sm text-left outline-none focus:ring-1 transition-colors"
            :style="{
              backgroundColor: 'var(--theme-cell)',
              borderColor: 'var(--theme-border)',
              color: 'var(--theme-text)',
              '--tw-ring-color': 'var(--theme-primary)',
            }"
          >
            {{ formatDateDisplay(localDoneDate) || '选择日期' }}
          </button>
          <MiniCalendar
            v-model:current-date="doneDateValue"
            :visible="showDoneDateCalendar"
            :centered="calendarCentered"
            @select="handleDoneDateSelect"
            @close="showDoneDateCalendar = false"
          />
        </div>
      </div>
    </div>

    <div v-if="showFatherTask" class="flex flex-col gap-1 shrink-0">
      <label class="flex items-center gap-1.5 font-medium" :style="{ color: 'var(--theme-text-muted)' }">
        <Link2 class="w-3 h-3" />
        <span>关联任务</span>
      </label>

      <div v-if="!editableFatherTask" class="schedule-input w-full px-2 py-1.5 rounded text-sm"
        :style="{
          backgroundColor: 'var(--theme-cell)',
          borderColor: 'var(--theme-border)',
          color: 'var(--theme-text)',
        }">
        {{ selectedTaskName || '无' }}
      </div>

      <div v-else class="relative">
        <div v-if="localFatherTaskId" class="schedule-input flex items-center gap-1 px-2 py-1.5 rounded text-sm"
          :style="{
            backgroundColor: 'var(--theme-cell)',
            borderColor: 'var(--theme-border)',
            color: 'var(--theme-text)',
          }">
          <span class="flex-1 truncate">{{ selectedTaskName }}</span>
          <button @click="clearTask" class="w-4 h-4 flex items-center justify-center rounded hover:bg-black/10 dark:hover:bg-white/10 transition-colors" :style="{ color: 'var(--theme-text-muted)' }">
            <X class="w-3 h-3" />
          </button>
        </div>
        <input
          v-else
          v-model="searchKeyword"
          type="text"
          placeholder="搜索主任务..."
          class="schedule-input w-full px-2 py-1.5 rounded text-sm outline-none focus:ring-1 transition-colors"
          :style="{
            backgroundColor: 'var(--theme-cell)',
            borderColor: 'var(--theme-border)',
            color: 'var(--theme-text)',
            '--tw-ring-color': 'var(--theme-primary)',
          }"
          @focus="handleTaskInputFocus"
          @blur="handleTaskInputBlur"
        />

        <div
          v-if="showTaskDropdown && filteredTasks.length > 0"
          class="absolute z-[60] mt-1 w-full max-h-36 overflow-y-auto rounded backdrop-blur-sm shadow-lg py-1"
          :style="{
            backgroundColor: 'var(--theme-cell)',
            borderColor: 'var(--theme-border)',
            border: '1px solid var(--theme-border)',
          }">
          <button
            v-for="task in filteredTasks"
            :key="task.id"
            type="button"
            @mousedown.prevent="selectTask(task)"
            class="w-full px-2 py-1.5 text-left text-sm transition-colors flex items-center gap-2 hover:bg-black/5 dark:hover:bg-white/5"
            :class="{ 'opacity-50': task.is_done }"
            :style="{ color: 'var(--theme-text)' }">
            <span class="w-1.5 h-1.5 rounded-full shrink-0" :class="task.is_done ? 'bg-green-500' : 'bg-blue-500'"></span>
            <span class="truncate flex-1" :class="{ 'line-through': task.is_done }">{{ task.content }}</span>
          </button>
        </div>

        <div
          v-if="showTaskDropdown && searchKeyword && filteredTasks.length === 0"
          class="absolute z-10 mt-1 w-full rounded backdrop-blur-sm shadow-lg p-1"
          :style="{
            backgroundColor: 'var(--theme-cell)',
            borderColor: 'var(--theme-border)',
            border: '1px solid var(--theme-border)',
          }">
          <button
            @mousedown.prevent="handleCreateNewTask"
            class="w-full flex items-center gap-1.5 px-2 py-1.5 text-sm rounded transition-colors text-left hover:bg-black/5 dark:hover:bg-white/5"
            :style="{ color: 'var(--theme-primary)' }"
          >
            <Plus class="w-3 h-3" />
            <span class="truncate">创建新任务: "{{ searchKeyword }}"</span>
          </button>
        </div>
      </div>
    </div>

    <div class="flex flex-col gap-1 flex-1 min-h-0">
      <label class="flex items-center gap-1.5 font-medium shrink-0" :style="{ color: 'var(--theme-text-muted)' }">
        <FileText class="w-3 h-3" />
        <span>描述</span>
      </label>
      <textarea
        v-model="localDescription"
        placeholder="添加描述..."
        class="schedule-input w-full flex-1 h-full rounded px-2 py-1.5 text-sm leading-relaxed outline-none resize-none focus:ring-1 transition-colors"
        :style="{
          backgroundColor: 'var(--theme-cell)',
          borderColor: 'var(--theme-border)',
          color: 'var(--theme-text)',
          '--tw-ring-color': 'var(--theme-primary)',
        }"
      ></textarea>
    </div>

    <div class="flex justify-between items-center gap-3 pt-2 mt-auto shrink-0">
      <button
        @click="handleCancel"
        class="w-10 h-10 flex items-center justify-center rounded-xl transition-all"
        :style="{ color: 'var(--theme-text-muted)' }"
        style="background-color: transparent;"
        @mouseenter="(e) => (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--theme-cell)'"
        @mouseleave="(e) => (e.currentTarget as HTMLElement).style.backgroundColor = 'transparent'"
        title="取消"
      >
        <X class="w-5 h-5" />
      </button>

      <button
        @click="handleSave"
        class="flex-1 h-10 flex items-center justify-center gap-1.5 rounded-xl text-white transition-opacity font-medium shadow-sm active:scale-[0.98]"
        :style="{ backgroundColor: 'var(--theme-primary)' }"
        @mouseenter="(e) => (e.currentTarget as HTMLElement).style.opacity = '0.9'"
        @mouseleave="(e) => (e.currentTarget as HTMLElement).style.opacity = '1'"
      >
        <Check class="w-4 h-4" />
        <span>保存</span>
      </button>
    </div>

  </div>
</template>

<style scoped>
.schedule-input {
  border: 1px solid var(--theme-border);
}
</style>
