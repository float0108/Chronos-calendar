<script setup lang="ts">
import { Settings, LogOut, Upload, FileDown, FolderDown, Cloud, Eye, EyeOff, CalendarPlus, Minimize } from 'lucide-vue-next';

defineProps<{
  visible: boolean;
  hideWeekends: boolean;
  isLocked: boolean;
}>();

const emit = defineEmits<{
  (e: 'settings'): void;
  (e: 'hide'): void;
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

function handleHide() {
  emit('hide');
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
      class="dropdown-menu fixed rounded-lg shadow-lg overflow-hidden z-50"
      :style="{ top: '60px', right: '16px' }"
      @mousedown.stop
      @click.stop
    >
    <button
      @click="handleSettings"
      class="menu-item w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
    >
      <Settings class="w-4 h-4" />
      <span>设置</span>
    </button>

    <div class="menu-divider"></div>

    <button
      @click="handleOpenBatchTask"
      class="menu-item w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
      :class="{ 'opacity-50 pointer-events-none': isLocked }"
      :disabled="isLocked"
    >
      <CalendarPlus class="w-4 h-4" />
      <span>批量添加任务</span>
    </button>

    <button
      @click="handleToggleWeekends"
      class="menu-item w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
    >
      <EyeOff v-if="hideWeekends" class="w-4 h-4" />
      <Eye v-else class="w-4 h-4" />
      <span>{{ hideWeekends ? '显示周末' : '隐藏周末' }}</span>
    </button>

    <div class="menu-divider"></div>

    <!-- 导出子菜单 -->
    <div class="menu-label px-3 py-1.5 text-xs">导出</div>
    <button
      @click="handleExportBackup"
      class="menu-item w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
    >
      <FileDown class="w-4 h-4" />
      <span>导出备份</span>
    </button>
    <button
      @click="handleExportZip"
      class="menu-item w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
    >
      <FolderDown class="w-4 h-4" />
      <span>导出压缩包</span>
    </button>

    <div class="menu-divider"></div>

    <!-- 导入 -->
    <button
      @click="handleImportBackup"
      class="menu-item w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
    >
      <Upload class="w-4 h-4" />
      <span>导入备份</span>
    </button>

    <div class="menu-divider"></div>

    <!-- 同步（预留） -->
    <button
      @click="handleSync"
      class="menu-item-disabled w-full flex items-center gap-2 px-3 py-2 text-sm cursor-not-allowed transition-colors"
      disabled
    >
      <Cloud class="w-4 h-4" />
      <span>同步 (开发中)</span>
    </button>

    <div class="menu-divider"></div>

    <button
      @click="handleHide"
      class="menu-item w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
    >
      <Minimize class="w-4 h-4" />
      <span>隐藏主窗口</span>
    </button>

    <button
      @click="handleQuit"
      class="menu-item-quit w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
    >
      <LogOut class="w-4 h-4" />
      <span>退出</span>
    </button>
  </div>
  </Teleport>
</template>

<style scoped>
.dropdown-menu {
  background-color: var(--solid-bg);
  border: 1px solid var(--border-light);
}

.menu-item {
  color: var(--text-primary);
}

.menu-item:hover:not(:disabled) {
  background-color: var(--hover-bg);
}

.menu-item-disabled {
  color: var(--text-muted);
}

.menu-divider {
  height: 1px;
  background-color: var(--border-light);
}

.menu-label {
  color: var(--text-muted);
  background-color: var(--hover-bg);
}

.menu-item-quit {
  color: #ef4444;
}

.menu-item-quit:hover {
  background-color: rgba(239, 68, 68, 0.1);
}
</style>
