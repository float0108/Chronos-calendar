// composables/useNotification.ts
import { ref } from 'vue';

export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface Notification {
  id: string;
  message: string;
  type: NotificationType;
  duration: number;
}

// Module-level singleton for backward compatibility with useToast
const notifications = ref<Notification[]>([]);

/**
 * 通知提示组合式函数
 *
 * 职责：
 * - 管理通知列表
 * - 显示/隐藏通知
 * - 自动消失
 *
 * @deprecated Use show/hide/clear methods directly on this module
 *             or create local refs with the composable function
 */
export function useNotification() {
  /**
   * 显示通知
   * @param message 通知内容
   * @param type 通知类型
   * @param duration 显示时长（毫秒），0 表示不自动消失
   */
  function show(
    message: string,
    type: NotificationType = 'info',
    duration = 3000
  ) {
    const id = Math.random().toString(36).substring(2, 11);
    notifications.value.push({ id, message, type, duration });

    if (duration > 0) {
      setTimeout(() => {
        dismiss(id);
      }, duration);
    }
  }

  /**
   * 关闭指定通知
   */
  function dismiss(id: string) {
    notifications.value = notifications.value.filter(n => n.id !== id);
  }

  /**
   * 清除所有通知
   */
  function clear() {
    notifications.value = [];
  }

  /**
   * 快捷方法：成功提示
   */
  function success(message: string, duration?: number) {
    show(message, 'success', duration);
  }

  /**
   * 快捷方法：错误提示
   */
  function error(message: string, duration?: number) {
    show(message, 'error', duration);
  }

  /**
   * 快捷方法：警告提示
   */
  function warning(message: string, duration?: number) {
    show(message, 'warning', duration);
  }

  /**
   * 快捷方法：信息提示
   */
  function info(message: string, duration?: number) {
    show(message, 'info', duration);
  }

  return {
    notifications,
    show,
    dismiss,
    clear,
    success,
    error,
    warning,
    info,
  };
}

// Backward compatibility alias for useToast
export function useToast() {
  const { notifications, show, dismiss } = useNotification();

  function showToast(message: string, type: NotificationType = 'info', duration = 3000) {
    show(message, type, duration);
  }

  function showError(message: string, duration = 4000) {
    show(message, 'error', duration);
  }

  function showSuccess(message: string, duration = 3000) {
    show(message, 'success', duration);
  }

  function showInfo(message: string, duration = 3000) {
    show(message, 'info', duration);
  }

  function removeToast(id: number) {
    const strId = String(id);
    dismiss(strId);
  }

  return {
    toasts: notifications,
    showToast,
    showError,
    showSuccess,
    showInfo,
    removeToast,
  };
}