import { ref } from 'vue';

type ToastType = 'error' | 'success' | 'info';

interface ToastMessage {
  id: number;
  message: string;
  type: ToastType;
}

const toasts = ref<ToastMessage[]>([]);
let toastId = 0;

export function useToast() {
  function showToast(message: string, type: ToastType = 'info', duration = 3000) {
    const id = ++toastId;
    toasts.value.push({ id, message, type });
    
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }

  function showError(message: string, duration = 4000) {
    showToast(message, 'error', duration);
  }

  function showSuccess(message: string, duration = 3000) {
    showToast(message, 'success', duration);
  }

  function showInfo(message: string, duration = 3000) {
    showToast(message, 'info', duration);
  }

  function removeToast(id: number) {
    const index = toasts.value.findIndex(t => t.id === id);
    if (index > -1) {
      toasts.value.splice(index, 1);
    }
  }

  return {
    toasts,
    showToast,
    showError,
    showSuccess,
    showInfo,
    removeToast,
  };
}
