import { ref, nextTick } from 'vue';

export interface EditLine {
  id?: number;
  text: string;
  done: boolean;
}

interface EditHistory {
  lines: EditLine[];
  activeLineIndex: number | null;
  cursorPosition: number;
}

export function useEditHistory(maxHistory: number = 50) {
  const undoHistory = ref<EditHistory[]>([]);
  const redoHistory = ref<EditHistory[]>([]);
  let lastSaveTime = 0;
  const saveDelay = 500; // 500ms 防抖

  /**
   * 保存当前编辑状态到历史
   */
  function saveHistory(
    lines: EditLine[],
    activeLineIndex: number | null,
    textareaRefs: (HTMLTextAreaElement | null)[]
  ): void {
    const textarea = activeLineIndex !== null ? textareaRefs[activeLineIndex] : null;
    const cursorPosition = textarea ? textarea.selectionStart : 0;

    undoHistory.value.push({
      lines: JSON.parse(JSON.stringify(lines)),
      activeLineIndex,
      cursorPosition,
    });

    if (undoHistory.value.length > maxHistory) {
      undoHistory.value.shift();
    }

    // 清空重做历史
    redoHistory.value = [];
  }

  /**
   * 防抖保存历史（用于文本输入）
   */
  function saveHistoryDebounced(
    lines: EditLine[],
    activeLineIndex: number | null,
    textareaRefs: (HTMLTextAreaElement | null)[]
  ): void {
    const now = Date.now();
    if (now - lastSaveTime > saveDelay) {
      saveHistory(lines, activeLineIndex, textareaRefs);
      lastSaveTime = now;
    }
  }

  /**
   * 撤销
   */
  function handleEditUndo(
    lines: EditLine[],
    activeLineIndex: number | null,
    textareaRefs: (HTMLTextAreaElement | null)[],
    onRestore: (history: EditHistory) => void
  ): void {
    if (undoHistory.value.length === 0) return;

    const textarea = activeLineIndex !== null ? textareaRefs[activeLineIndex] : null;
    const cursorPosition = textarea ? textarea.selectionStart : 0;

    // 保存当前状态到重做历史
    redoHistory.value.push({
      lines: JSON.parse(JSON.stringify(lines)),
      activeLineIndex,
      cursorPosition,
    });

    // 恢复上一个状态
    const history = undoHistory.value.pop()!;

    // 通过回调恢复状态
    onRestore(history);

    // 恢复光标位置
    nextTick(() => {
      if (history.activeLineIndex !== null) {
        const textarea = textareaRefs[history.activeLineIndex];
        if (textarea) {
          textarea.focus();
          textarea.setSelectionRange(history.cursorPosition, history.cursorPosition);
        }
      }
    });
  }

  /**
   * 重做
   */
  function handleEditRedo(
    lines: EditLine[],
    activeLineIndex: number | null,
    textareaRefs: (HTMLTextAreaElement | null)[],
    onRestore: (history: EditHistory) => void
  ): void {
    if (redoHistory.value.length === 0) return;

    const textarea = activeLineIndex !== null ? textareaRefs[activeLineIndex] : null;
    const cursorPosition = textarea ? textarea.selectionStart : 0;

    // 保存当前状态到撤销历史
    undoHistory.value.push({
      lines: JSON.parse(JSON.stringify(lines)),
      activeLineIndex,
      cursorPosition,
    });

    // 恢复下一个状态
    const history = redoHistory.value.pop()!;

    // 通过回调恢复状态
    onRestore(history);

    // 恢复光标位置
    nextTick(() => {
      if (history.activeLineIndex !== null) {
        const textarea = textareaRefs[history.activeLineIndex];
        if (textarea) {
          textarea.focus();
          textarea.setSelectionRange(history.cursorPosition, history.cursorPosition);
        }
      }
    });
  }

  /**
   * 重置历史记录
   */
  function resetHistory(): void {
    undoHistory.value = [];
    redoHistory.value = [];
    lastSaveTime = 0;
  }

  return {
    undoHistory,
    redoHistory,
    saveHistory,
    saveHistoryDebounced,
    handleEditUndo,
    handleEditRedo,
    resetHistory,
  };
}
