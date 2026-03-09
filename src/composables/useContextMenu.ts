import { ref, computed } from 'vue';
import { colorOptions } from '../constants';

/**
 * 上下文菜单状态管理
 */
export function useContextMenu() {
  const contextMenu = ref<{
    show: boolean;
    x: number;
    y: number;
    date: string;
  }>({
    show: false,
    x: 0,
    y: 0,
    date: '',
  });

  const contextMenuStyle = computed(() => {
    const menuWidth = 100;
    const menuHeight = 280;
    let x = contextMenu.value.x;
    let y = contextMenu.value.y;

    if (typeof window !== 'undefined') {
      if (x + menuWidth > window.innerWidth) {
        x = window.innerWidth - menuWidth - 10;
      }
      if (y + menuHeight > window.innerHeight) {
        y = window.innerHeight - menuHeight - 10;
      }
    }

    return { left: x + 'px', top: y + 'px' };
  });

  /**
   * 显示上下文菜单
   */
  function showContextMenu(event: MouseEvent, date: string): void {
    contextMenu.value = {
      show: true,
      x: event.clientX,
      y: event.clientY,
      date,
    };
  }

  /**
   * 隐藏上下文菜单
   */
  function hideContextMenu(): void {
    contextMenu.value.show = false;
  }

  /**
   * 获取当前选中的日期
   */
  function getSelectedDate(): string {
    return contextMenu.value.date;
  }

  /**
   * 获取颜色选项
   */
  function getColorOptions() {
    return colorOptions;
  }

  return {
    contextMenu,
    contextMenuStyle,
    showContextMenu,
    hideContextMenu,
    getSelectedDate,
    getColorOptions,
  };
}
