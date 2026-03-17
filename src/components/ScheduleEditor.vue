<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue';
import { Calendar, CheckCircle, Link2, FileText, X } from 'lucide-vue-next';
import { loadMainTasks, type MainTask } from '../composables/db';

const props = defineProps<{
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
}>();

const emit = defineEmits<{
  (e: 'update:content', value: string): void;
  (e: 'update:description', value: string): void;
  (e: 'update:createDate', value: string): void;
  (e: 'update:doneDate', value: string): void;
  (e: 'update:fatherTaskId', value: number | null): void;
}>();

// 本地状态
const localContent = ref(props.content || '');
const localDescription = ref(props.description || '');
const localCreateDate = ref(props.createDate || '');
const localDoneDate = ref(props.doneDate || '');
const localFatherTaskId = ref<number | null>(props.fatherTaskId || null);

// 主任务列表
const mainTasks = ref<MainTask[]>([]);
const searchKeyword = ref('');
const showTaskDropdown = ref(false);

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
watch(() => props.fatherTaskId, (val) => { localFatherTaskId.value = val || null; });

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
  localFatherTaskId.value = task?.id || null;
  searchKeyword.value = '';
  showTaskDropdown.value = false;
}

function clearTask() {
  localFatherTaskId.value = null;
  searchKeyword.value = '';
}

onMounted(() => {
  if (props.showFatherTask) {
    loadTasks();
  }
});

// 暴露方法供父组件调用
defineExpose({
  loadTasks
});
</script>

<template>
  <div class="schedule-editor flex flex-col gap-4">
    <!-- 内容/标题 -->
    <div v-if="showContent" class="flex flex-col gap-1.5">
      <label class="flex items-center gap-2 text-[12px] font-medium"
        :style="{ color: 'var(--theme-text-muted)' }">
        <span>标题</span>
      </label>
      <input
        v-model="localContent"
        type="text"
        placeholder="输入标题..."
        class="w-full px-3 py-2 rounded-lg text-[13px] bg-black/5 dark:bg-white/5 border border-[var(--theme-border)] outline-none focus:ring-1 focus:ring-[var(--theme-primary)]"
        :style="{ color: 'var(--theme-text)' }"
      />
    </div>

    <!-- 创建日期 -->
    <div v-if="showCreateDate" class="flex flex-col gap-1.5">
      <label class="flex items-center gap-2 text-[12px] font-medium"
        :style="{ color: 'var(--theme-text-muted)' }">
        <Calendar class="w-3.5 h-3.5" />
        <span>创建日期</span>
      </label>
      <input type="date"
        v-model="localCreateDate"
        class="w-full px-3 py-2 rounded-lg text-[13px] bg-black/5 dark:bg-white/5 border border-[var(--theme-border)] outline-none focus:ring-1 focus:ring-[var(--theme-primary)]"
        :style="{ color: 'var(--theme-text)' }" />
    </div>

    <!-- 完成日期 -->
    <div v-if="showDoneDate" class="flex flex-col gap-1.5">
      <label class="flex items-center gap-2 text-[12px] font-medium"
        :style="{ color: 'var(--theme-text-muted)' }">
        <CheckCircle class="w-3.5 h-3.5" />
        <span>完成日期</span>
      </label>
      <input type="date"
        v-model="localDoneDate"
        class="w-full px-3 py-2 rounded-lg text-[13px] bg-black/5 dark:bg-white/5 border border-[var(--theme-border)] outline-none focus:ring-1 focus:ring-[var(--theme-primary)]"
        :style="{ color: 'var(--theme-text)' }" />
    </div>

    <!-- 关联主任务 -->
    <div v-if="showFatherTask" class="flex flex-col gap-1.5">
      <label class="flex items-center gap-2 text-[12px] font-medium"
        :style="{ color: 'var(--theme-text-muted)' }">
        <Link2 class="w-3.5 h-3.5" />
        <span>关联任务</span>
      </label>

      <!-- 只读模式 -->
      <div v-if="!editableFatherTask" class="w-full px-3 py-2 rounded-lg text-[13px] bg-black/5 dark:bg-white/5 border border-[var(--theme-border)]"
        :style="{ color: 'var(--theme-text)' }">
        {{ selectedTaskName || '无' }}
      </div>

      <!-- 可编辑模式 -->
      <div v-else class="relative">
        <div v-if="localFatherTaskId" class="flex items-center gap-1 px-3 py-2 rounded-lg text-[13px] bg-black/5 dark:bg-white/5 border border-[var(--theme-border)]">
          <span class="flex-1 truncate" :style="{ color: 'var(--theme-text)' }">{{ selectedTaskName }}</span>
          <button @click="clearTask" class="w-4 h-4 flex items-center justify-center rounded hover:opacity-70" :style="{ color: 'var(--theme-text-muted)' }">
            <X class="w-3 h-3" />
          </button>
        </div>
        <input
          v-else
          v-model="searchKeyword"
          type="text"
          placeholder="搜索主任务..."
          class="w-full px-3 py-2 rounded-lg text-[13px] bg-black/5 dark:bg-white/5 border border-[var(--theme-border)] outline-none focus:ring-1 focus:ring-[var(--theme-primary)]"
          :style="{ color: 'var(--theme-text)' }"
          @focus="handleTaskInputFocus"
          @blur="handleTaskInputBlur"
        />
        <!-- 下拉列表 -->
        <div
          v-if="showTaskDropdown && filteredTasks.length > 0"
          class="absolute z-[60] mt-1 w-full max-h-48 overflow-y-auto rounded-lg bg-white/95 dark:bg-gray-800/95 backdrop-blur-sm border border-gray-200/60 dark:border-white/10 shadow-lg">
          <button
            v-for="task in filteredTasks"
            :key="task.id"
            type="button"
            @mousedown.prevent="selectTask(task)"
            class="w-full px-3 py-2 text-left text-[13px] hover:bg-blue-50/50 dark:hover:bg-blue-500/10 transition-colors flex items-center gap-2"
            :class="{ 'opacity-50': task.is_done }">
            <span class="w-1.5 h-1.5 rounded-full shrink-0" :class="task.is_done ? 'bg-green-500' : 'bg-blue-500'"></span>
            <span class="truncate flex-1" :class="{ 'line-through': task.is_done }">{{ task.content }}</span>
          </button>
        </div>
        <div
          v-if="showTaskDropdown && searchKeyword && filteredTasks.length === 0"
          class="absolute z-10 mt-1 w-full rounded-lg bg-white/95 dark:bg-gray-800/95 backdrop-blur-sm border border-gray-200/60 dark:border-white/10 shadow-lg px-3 py-2 text-[13px] text-gray-500">
          无匹配任务
        </div>
      </div>
    </div>

    <!-- 描述 -->
    <div class="flex flex-col gap-1.5">
      <label class="flex items-center gap-2 text-[12px] font-medium"
        :style="{ color: 'var(--theme-text-muted)' }">
        <FileText class="w-3.5 h-3.5" />
        <span>描述</span>
      </label>
      <textarea
        v-model="localDescription"
        placeholder="添加描述..."
        class="w-full min-h-[80px] bg-transparent border border-[var(--theme-border)] rounded-lg px-3 py-2 text-[13px] leading-relaxed outline-none resize-none focus:ring-1 focus:ring-[var(--theme-primary)]"
        :style="{ color: 'var(--theme-text)' }"
      ></textarea>
    </div>
  </div>
</template>
