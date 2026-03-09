/**
 * 数据库操作 composable
 * 向后兼容的统一入口
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
  updateScheduleColor,
  getCellColor,
  loadAllSchedules,
  importSchedules,
  importCellColors,
  clearAllData,
} from './db';

// 保持 db ref 的导出（向后兼容）
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
    updateScheduleColor,
    getCellColor,
    loadAllSchedules,
    importSchedules,
    importCellColors,
    clearAllData,
  };
}

// 直接导出函数供不需要响应式的场景使用
export type { Schedule };
