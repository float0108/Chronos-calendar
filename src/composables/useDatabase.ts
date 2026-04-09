/**
 * 数据库操作 composable
 *
 * 所有数据库操作通过 Tauri command 调用后端
 */
import type { Schedule } from '../types';
import {
  loadSchedules,
  loadTodoSchedules,
  loadDoneSchedules,
  saveSchedule,
  deleteSchedule,
  deleteSchedulesByDate,
  toggleScheduleStatus,
  updateScheduleDescription,
  updateScheduleContent,
  updateScheduleDate,
  updateScheduleFatherTask,
  updateScheduleColor,
  getCellColor,
  loadAllSchedules,
  importSchedules,
  importCellColors,
  clearAllData,
  exportAllData,
  importAndMergeData,
  clearAllTables,
} from './db';

export function useDatabase() {
  return {
    // 不再需要前端初始化数据库，后端在启动时自动初始化
    initDatabase: async () => { /* no-op */ },
    loadSchedules,
    loadTodoSchedules,
    loadDoneSchedules,
    saveSchedule,
    deleteSchedule,
    deleteSchedulesByDate,
    toggleScheduleStatus,
    updateScheduleDescription,
    updateScheduleContent,
    updateScheduleDate,
    updateScheduleFatherTask,
    updateScheduleColor,
    getCellColor,
    loadAllSchedules,
    importSchedules,
    importCellColors,
    clearAllData,
    exportAllData,
    importAndMergeData,
    clearAllTables,
  };
}

// 直接导出函数供不需要响应式的场景使用
export type { Schedule };
