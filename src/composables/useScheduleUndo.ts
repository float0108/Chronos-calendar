import { ref, type Ref } from 'vue';
import type { Schedule } from '../types';
import { useToast } from './useToast';

export interface UndoAction {
  type: 'toggleDone' | 'deleteSchedule' | 'editSchedule' | 'addSchedule' | 'updateLines';
  data: any;
  timestamp: number;
}

export function useScheduleUndo(
  schedules: Ref<Map<string, Schedule[]>>,
  toggleScheduleStatus: (id: number, isDone: boolean) => Promise<void>,
  refreshSchedules: () => Promise<void>,
  updateScheduleLines: (date: string, lines: { text: string; done: boolean }[]) => Promise<void>,
  saveSchedule: (date: string, content: string, isDone?: boolean, doneDate?: string, description?: string) => Promise<void>
) {
  const { showSuccess, showError } = useToast();
  const history = ref<UndoAction[]>([]);
  const redoHistory = ref<UndoAction[]>([]);
  const maxHistory = 50;

  function pushAction(action: UndoAction) {
    history.value.push(action);
    if (history.value.length > maxHistory) {
      history.value.shift();
    }
    // 清空重做历史
    redoHistory.value = [];
  }

  function popAction(): UndoAction | undefined {
    const action = history.value.pop();
    if (action) {
      redoHistory.value.push(action);
    }
    return action;
  }

  function pushRedo(action: UndoAction) {
    redoHistory.value.push(action);
    if (redoHistory.value.length > maxHistory) {
      redoHistory.value.shift();
    }
  }

  function popRedo(): UndoAction | undefined {
    return redoHistory.value.pop();
  }

  function clearHistory() {
    history.value = [];
    redoHistory.value = [];
  }

  function canUndo(): boolean {
    return history.value.length > 0;
  }

  function canRedo(): boolean {
    return redoHistory.value.length > 0;
  }

  async function handleToggleDone(schedule: Schedule): Promise<void> {
    // 保存撤销状态
    pushAction({
      type: 'toggleDone',
      data: {
        id: schedule.id,
        previousState: schedule.is_done,
      },
      timestamp: Date.now(),
    });

    // 切换完成状态（传入切换后的状态，而不是当前状态）
    await toggleScheduleStatus(schedule.id!, !schedule.is_done);
    await refreshSchedules();
  }

  async function handleUndo(): Promise<void> {
    const action = popAction();
    if (!action) return;

    try {
      switch (action.type) {
        case 'toggleDone': {
          const { id, previousState } = action.data;
          await toggleScheduleStatus(id, !previousState);
          await refreshSchedules();
          showSuccess('已撤销：切换完成状态');
          break;
        }
        case 'updateLines': {
          const { date, previousLines } = action.data;
          // 获取当前状态用于重做
          const currentSchedules = schedules.value.get(date) || [];
          const currentLines = currentSchedules
            .filter(s => s.id !== -1 && s.content.trim() !== '')
            .map(s => ({ text: s.content.trim(), done: !!s.is_done }));
          // 保存当前状态到重做历史
          pushRedo({
            type: 'updateLines',
            data: { date, previousLines: currentLines },
            timestamp: Date.now(),
          });
          await updateScheduleLines(date, previousLines);
          await refreshSchedules();
          showSuccess('已撤销：编辑操作');
          break;
        }
        case 'deleteSchedule': {
          const { date, previousSchedules } = action.data;
          // 恢复删除的日程
          for (const schedule of previousSchedules) {
            await saveSchedule(date, schedule.content, schedule.is_done, schedule.done_date, schedule.description);
          }
          await refreshSchedules();
          showSuccess('已撤销：删除操作');
          break;
        }
      }
    } catch (error) {
      console.error('Undo failed:', error);
      showError('撤销失败');
    }
  }

  async function handleRedo(): Promise<void> {
    const action = popRedo();
    if (!action) return;

    try {
      switch (action.type) {
        case 'updateLines': {
          const { date, previousLines } = action.data;
          await updateScheduleLines(date, previousLines);
          await refreshSchedules();
          showSuccess('已重做：编辑操作');
          break;
        }
        default:
          console.warn('Unsupported redo action type:', action.type);
      }
    } catch (error) {
      console.error('Redo failed:', error);
      showError('重做失败');
    }
  }

  return {
    pushAction,
    popAction,
    pushRedo,
    popRedo,
    clearHistory,
    canUndo,
    canRedo,
    handleToggleDone,
    handleUndo,
    handleRedo,
  };
}
