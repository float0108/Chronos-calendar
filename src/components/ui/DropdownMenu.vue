<script setup lang="ts">
import { Settings, LogOut, Download, Upload } from 'lucide-vue-next';

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'settings'): void;
  (e: 'quit'): void;
  (e: 'export'): void;
  (e: 'import'): void;
}>();

function handleSettings() {
  emit('settings');
}

function handleQuit() {
  emit('quit');
}

function handleExport() {
  emit('export');
}

function handleImport() {
  emit('import');
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
      @click="handleExport"
      class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-900 hover:bg-gray-100 transition-colors"
    >
      <Download class="w-4 h-4" />
      <span>导出</span>
    </button>

    <button
      @click="handleImport"
      class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-900 hover:bg-gray-100 transition-colors"
    >
      <Upload class="w-4 h-4" />
      <span>导入</span>
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
