/**
 * 数据库操作 composable
 */
import { ref } from 'vue';
import type { Schedule } from '../types';
import {
  initDatabase as dbInit,
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

const db = ref<unknown>(null);

export function useDatabase() {
  return {
    db,
    initDatabase: dbInit,
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
