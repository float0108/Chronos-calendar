import { ref } from 'vue';

export interface UndoAction {
  type: 'toggleDone' | 'deleteSchedule' | 'editSchedule' | 'addSchedule' | 'updateLines';
  data: any;
  timestamp: number;
}

const history = ref<UndoAction[]>([]);
const maxHistory = 50;

export function useUndoHistory() {
  function pushAction(action: UndoAction) {
    history.value.push(action);
    if (history.value.length > maxHistory) {
      history.value.shift();
    }
  }

  function popAction(): UndoAction | undefined {
    return history.value.pop();
  }

  function clearHistory() {
    history.value = [];
  }

  function canUndo(): boolean {
    return history.value.length > 0;
  }

  return {
    history,
    pushAction,
    popAction,
    clearHistory,
    canUndo,
  };
}
