// composables/useAutoSave.ts
import { ref } from 'vue';

/**
 * 自动保存组合式函数
 *
 * 职责：
 * - 防抖处理（Debounce）
 * - 防止重复保存
 * - 错误处理
 *
 * @param delay 防抖延迟（毫秒），默认 1000ms
 */
export function useAutoSave(delay = 1000) {
  let timer: ReturnType<typeof setTimeout> | null = null;
  const isSaving = ref(false);
  const error = ref<string | null>(null);

  /**
   * 执行自动保存
   * @param saveFn 保存函数，应返回 Promise
   */
  async function save(saveFn: () => Promise<void>) {
    if (timer) {
      clearTimeout(timer);
    }

    timer = setTimeout(async () => {
      if (isSaving.value) return;

      isSaving.value = true;
      error.value = null;

      try {
        await saveFn();
      } catch (err) {
        error.value = err instanceof Error ? err.message : 'Save failed';
        console.error('Auto-save error:', err);
      } finally {
        isSaving.value = false;
      }
    }, delay);
  }

  /**
   * 取消待执行的保存
   */
  function cancel() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  }

  /**
   * 清理资源
   */
  function dispose() {
    cancel();
  }

  return {
    isSaving,
    error,
    save,
    cancel,
    dispose,
  };
}