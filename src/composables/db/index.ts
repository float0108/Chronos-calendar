// 数据库模块统一导出
export { getDatabase, initDatabase } from './connection';
export type { Schedule } from '../../types';
export {
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
  loadSchedulesByFatherTask,
  saveSubTask,
} from './schedule';
export {
  updateScheduleColor,
  getCellColor,
  loadCellColorMap,
  loadAllSchedules,
  importSchedules,
  importCellColors,
  clearAllData,
} from './import';
export {
  loadMainTasks,
  searchMainTasks,
  saveMainTask,
  updateMainTaskContent,
  updateMainTaskDescription,
  updateMainTaskCreateDate,
  updateMainTaskDoneDate,
  toggleMainTaskStatus,
  updateMainTaskPriority,
  deleteMainTask,
  type MainTask,
} from './mainTask';
export {
  loadNotes,
  searchNotes,
  getNote,
  createNote,
  updateNote,
  updateNoteTitle,
  updateNoteContent,
  updateNoteCreateDate,
  deleteNote,
  type Note,
} from './note';
export {
  exportAllData,
  importAndMergeData,
  clearAllTables,
  resetAutoIncrement,
  type BackupData,
} from './backup';
