<script setup lang="ts">
import { Settings, LogOut, Upload, FileDown, FolderDown, Cloud, Eye, EyeOff, CalendarPlus } from 'lucide-vue-next';

defineProps<{
  visible: boolean;
  hideWeekends: boolean;
  isLocked: boolean;
}>();

const emit = defineEmits<{
  (e: 'settings'): void;
  (e: 'quit'): void;
  (e: 'exportBackup'): void;
  (e: 'exportZip'): void;
  (e: 'importBackup'): void;
  (e: 'sync'): void;
  (e: 'toggleWeekends'): void;
  (e: 'openBatchTask'): void;
}>();

function handleSettings() {
  emit('settings');
}

function handleQuit() {
  emit('quit');
}

function handleExportBackup() {
  emit('exportBackup');
}

function handleExportZip() {
  emit('exportZip');
}

function handleImportBackup() {
  emit('importBackup');
}

function handleSync() {
  emit('sync');
}

function handleToggleWeekends() {
  emit('toggleWeekends');
}

function handleOpenBatchTask() {
  emit('openBatchTask');
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="dropdown-menu fixed bg-white rounded-lg border border-gray-200 shadow-lg overflow-hidden z-50"
      :style="{ top: '60px', right: '16px' }"
      @mousedown.stop
      @click.stop
    >
    <button
      @click="handleSettings"
      class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-900 hover:bg-gray-100 transition-colors"
    >
      <Settings class="w-4 h-4" />
      <span>设置</span>
    </button>

    <div class="h-px bg-gray-200"></div>

    <button
      @click="handleOpenBatchTask"
      class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-900 hover:bg-gray-100 transition-colors"
      :class="{ 'opacity-50 pointer-events-none': isLocked }"
      :disabled="isLocked"
    >
      <CalendarPlus class="w-4 h-4" />
      <span>批量添加任务</span>
    </button>

    <button
      @click="handleToggleWeekends"
      class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-900 hover:bg-gray-100 transition-colors"
    >
      <EyeOff v-if="hideWeekends" class="w-4 h-4" />
      <Eye v-else class="w-4 h-4" />
      <span>{{ hideWeekends ? '显示周末' : '隐藏周末' }}</span>
    </button>

    <div class="h-px bg-gray-200"></div>

    <!-- 导出子菜单 -->
    <div class="px-3 py-1.5 text-xs text-gray-500 bg-gray-50">导出</div>
    <button
      @click="handleExportBackup"
      class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-900 hover:bg-gray-100 transition-colors"
    >
      <FileDown class="w-4 h-4" />
      <span>导出备份</span>
    </button>
    <button
      @click="handleExportZip"
      class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-900 hover:bg-gray-100 transition-colors"
    >
      <FolderDown class="w-4 h-4" />
      <span>导出压缩包</span>
    </button>

    <div class="h-px bg-gray-200"></div>

    <!-- 导入 -->
    <button
      @click="handleImportBackup"
      class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-900 hover:bg-gray-100 transition-colors"
    >
      <Upload class="w-4 h-4" />
      <span>导入备份</span>
    </button>

    <div class="h-px bg-gray-200"></div>

    <!-- 同步（预留） -->
    <button
      @click="handleSync"
      class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-400 cursor-not-allowed transition-colors"
      disabled
    >
      <Cloud class="w-4 h-4" />
      <span>同步 (开发中)</span>
    </button>

    <div class="h-px bg-gray-200"></div>

    <button
      @click="handleQuit"
      class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-500 hover:bg-red-50 transition-colors"
    >
      <LogOut class="w-4 h-4" />
      <span>退出</span>
    </button>
  </div>
  </Teleport>
</template>
