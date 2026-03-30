<script setup lang="ts">
import { Settings, LogOut, Upload, FileDown, FolderDown, Cloud, Eye, EyeOff, CalendarPlus, Minimize } from 'lucide-vue-next';
import { computed } from 'vue';

defineProps<{
  visible: boolean;
  hideWeekends: boolean;
  isLocked: boolean;
}>();

// 检测当前主题模式
const isDarkMode = computed(() => {
  if (typeof document === 'undefined') return false;
  return document.documentElement.getAttribute('data-theme') === 'dark';
});

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
      :class="isDarkMode ? 'dark-mode' : 'light-mode'"
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
/* 亮色模式：白色背景，黑色文字 */
.dropdown-menu.light-mode {
  background-color: #ffffff;
  border: 1px solid #e5e7eb;
}

/* 暗色模式：黑色背景，白色文字 */
.dropdown-menu.dark-mode {
  background-color: #000000;
  border: 1px solid #374151;
}

/* 菜单项样式 - 亮色模式 */
.dropdown-menu.light-mode .menu-item {
  color: #000000;
}

.dropdown-menu.light-mode .menu-item:hover:not(:disabled) {
  background-color: #f3f4f6;
}

/* 菜单项样式 - 暗色模式 */
.dropdown-menu.dark-mode .menu-item {
  color: #ffffff;
}

.dropdown-menu.dark-mode .menu-item:hover:not(:disabled) {
  background-color: #374151;
}

/* 禁用项样式 */
.dropdown-menu.light-mode .menu-item-disabled {
  color: #9ca3af;
}

.dropdown-menu.dark-mode .menu-item-disabled {
  color: #6b7280;
}

/* 分割线样式 */
.dropdown-menu.light-mode .menu-divider {
  height: 1px;
  background-color: #e5e7eb;
}

.dropdown-menu.dark-mode .menu-divider {
  height: 1px;
  background-color: #374151;
}

/* 标签样式 */
.dropdown-menu.light-mode .menu-label {
  color: #6b7280;
  background-color: #f9fafb;
}

.dropdown-menu.dark-mode .menu-label {
  color: #9ca3af;
  background-color: #111827;
}

/* 退出按钮样式 */
.dropdown-menu.light-mode .menu-item-quit {
  color: #ef4444;
}

.dropdown-menu.light-mode .menu-item-quit:hover {
  background-color: #fef2f2;
}

.dropdown-menu.dark-mode .menu-item-quit {
  color: #f87171;
}

.dropdown-menu.dark-mode .menu-item-quit:hover {
  background-color: #7f1d1d;
}
</style>
