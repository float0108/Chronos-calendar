import { ref } from 'vue';

export interface UndoAction {
  type: 'toggleDone' | 'deleteSchedule' | 'editSchedule' | 'addSchedule' | 'updateLines';
  data: any;
  timestamp: number;
}

const history = ref<UndoAction[]>([]);
const redoHistory = ref<UndoAction[]>([]);
const maxHistory = 50;

export function useUndoHistory() {
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

  return {
    history,
    redoHistory,
    pushAction,
    popAction,
    pushRedo,
    popRedo,
    clearHistory,
    canUndo,
    canRedo,
  };
}
