<script setup lang="ts">
import { computed } from 'vue';
import { AlertCircle } from 'lucide-vue-next';

const props = defineProps<{
  visible: boolean;
  recordCount: number;
}>();

const emit = defineEmits<{
  (e: 'merge'): void;
  (e: 'overwrite'): void;
  (e: 'cancel'): void;
}>();

const recordText = computed(() => {
  return props.recordCount === 1 ? '1 条日程记录' : `${props.recordCount} 条日程记录`;
});

function handleMerge() {
  emit('merge');
}

function handleOverwrite() {
  emit('overwrite');
}

function handleCancel() {
  emit('cancel');
}
</script>

<template>
  <div
    v-if="visible"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50"
    @click.self="handleCancel"
  >
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 max-w-md w-full mx-4">
      <div class="flex items-center gap-3 mb-4">
        <AlertCircle class="w-6 h-6 text-blue-500" />
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
          导入日程数据
        </h3>
      </div>

      <p class="text-sm text-gray-600 dark:text-gray-300 mb-6">
        发现 CSV 文件包含 <span class="font-semibold text-blue-600">{{ recordText }}</span>，如何处理现有数据？
      </p>

      <div class="space-y-3">
        <button
          @click="handleMerge"
          class="w-full px-4 py-2.5 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors font-medium"
        >
          合并（保留现有数据）
        </button>

        <button
          @click="handleOverwrite"
          class="w-full px-4 py-2.5 bg-orange-500 text-white rounded-lg hover:bg-orange-600 transition-colors font-medium"
        >
          覆盖（替换现有数据）
        </button>

        <button
          @click="handleCancel"
          class="w-full px-4 py-2.5 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition-colors font-medium dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
        >
          取消
        </button>
      </div>

      <p class="text-xs text-gray-500 dark:text-gray-400 mt-4">
        提示：合并会将新数据添加到现有数据之后，覆盖会删除所有现有数据后再导入。
      </p>
    </div>
  </div>
</template>
