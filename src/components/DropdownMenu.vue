<script setup lang="ts">
import { Settings, LogOut } from 'lucide-vue-next';

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'settings'): void;
  (e: 'quit'): void;
}>();

function handleSettings() {
  emit('settings');
}

function handleQuit() {
  emit('quit');
}
</script>

<template>
  <div 
    v-if="visible"
    class="dropdown-menu absolute right-0 top-full mt-1 w-32 bg-white rounded-lg border border-gray-200 shadow-lg overflow-hidden z-50"
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
      @click="handleQuit"
      class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-500 hover:bg-red-50 transition-colors"
    >
      <LogOut class="w-4 h-4" />
      <span>退出</span>
    </button>
  </div>
</template>
