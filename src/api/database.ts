/**
 * 数据库 API 层
 *
 * 通过 Tauri command 调用后端数据库操作
 */

import { invoke } from '@tauri-apps/api/core';
import type { Schedule } from '../types';

// 重新导出 Schedule 类型供其他模块使用
export type { Schedule } from '../types';

// ========== Types ==========

export interface MainTask {
  id?: number;
  content: string;
  description?: string;
  is_done: boolean;
  priority: number;
  create_date: string;
  done_date?: string;
}

export interface Note {
  id?: number;
  title: string;
  content: string;
  create_date: string;
}

export interface CellMetadata {
  date: string;
  cell_color: string | null;
}

// ========== Backend Types (snake_case from Rust) ==========

interface BackendSchedule {
  id: number;
  create_date: string | null;
  content: string;
  is_done: boolean;
  priority: number;
  done_date: string | null;
  description: string | null;
  father_task: number | null;
}

interface BackendMainTask {
  id: number;
  content: string;
  description: string | null;
  is_done: boolean;
  priority: number;
  create_date: string;
  done_date: string | null;
}

interface BackendNote {
  id: number;
  title: string;
  content: string;
  create_date: string;
}

interface BackendBackupData {
  schedules: BackendSchedule[];
  main_tasks: BackendMainTask[];
  notes: BackendNote[];
  cell_metadata: CellMetadata[];
}

interface BackendImportStats {
  schedules: TableStats;
  main_tasks: TableStats;
  notes: TableStats;
  cell_metadata: TableStats;
}

export interface TableStats {
  inserted: number;
  updated: number;
}

// 兼容原有 ExportData 类型
export interface ExportData {
  schedules: Schedule[];
  mainTasks: MainTask[];
  notes: Note[];
  cellMetadata: { date: string; cell_color: string }[];
}

export interface ImportStats {
  schedules: TableStats;
  mainTasks: TableStats;
  notes: TableStats;
  cellMetadata: TableStats;
}

// ========== Helper Functions ==========

function convertSchedule(s: BackendSchedule): Schedule {
  return {
    id: s.id,
    create_date: s.create_date || '',
    content: s.content,
    is_done: s.is_done,
    priority: s.priority,
    done_date: s.done_date || undefined,
    description: s.description || undefined,
    father_task: s.father_task || undefined,
  };
}

function convertMainTask(t: BackendMainTask): MainTask {
  return {
    id: t.id,
    content: t.content,
    description: t.description || undefined,
    is_done: t.is_done,
    priority: t.priority,
    create_date: t.create_date,
    done_date: t.done_date || undefined,
  };
}

function convertNote(n: BackendNote): Note {
  return {
    id: n.id,
    title: n.title,
    content: n.content,
    create_date: n.create_date,
  };
}

function convertImportStats(stats: BackendImportStats): ImportStats {
  return {
    schedules: stats.schedules,
    mainTasks: stats.main_tasks,
    notes: stats.notes,
    cellMetadata: stats.cell_metadata,
  };
}

// ========== Schedule API ==========

export async function loadSchedules(startDate: string, endDate: string): Promise<Schedule[]> {
  const result = await invoke<BackendSchedule[]>('db_load_schedules', { startDate, endDate });
  return result.map(convertSchedule);
}

export async function loadTodoSchedules(startDate: string, endDate: string): Promise<Schedule[]> {
  const result = await invoke<BackendSchedule[]>('db_load_todo_schedules', { startDate, endDate });
  return result.map(convertSchedule);
}

export async function loadDoneSchedules(startDate: string, endDate: string): Promise<Schedule[]> {
  const result = await invoke<BackendSchedule[]>('db_load_done_schedules', { startDate, endDate });
  return result.map(convertSchedule);
}

export async function saveSchedule(
  createDate: string,
  content: string,
  isDone: boolean = false,
  doneDate?: string,
  description?: string,
  fatherTask?: number
): Promise<void> {
  await invoke('db_save_schedule', {
    createDate,
    content,
    isDone,
    doneDate,
    description,
    fatherTask,
  });
}

export async function deleteSchedule(id: number): Promise<void> {
  await invoke('db_delete_schedule', { id });
}

export async function deleteSchedulesByDate(date: string): Promise<void> {
  await invoke('db_delete_schedules_by_date', { date });
}

export async function toggleScheduleStatus(id: number, isDone: boolean): Promise<void> {
  await invoke('db_toggle_schedule_status', { id, isDone });
}

export async function updateScheduleContent(id: number, content: string): Promise<void> {
  await invoke('db_update_schedule_content', { id, content });
}

export async function updateScheduleDescription(id: number, description: string): Promise<void> {
  await invoke('db_update_schedule_description', { id, description: description || null });
}

export async function updateScheduleDate(
  id: number,
  field: 'create_date' | 'done_date',
  date: string
): Promise<void> {
  await invoke('db_update_schedule_date', { id, field, date });
}

export async function updateScheduleFatherTask(id: number, fatherTask: number | null): Promise<void> {
  await invoke('db_update_schedule_father_task', { id, fatherTask });
}

export async function loadSchedulesByFatherTask(fatherTaskId: number): Promise<Schedule[]> {
  const result = await invoke<BackendSchedule[]>('db_load_schedules_by_father_task', { fatherTaskId });
  return result.map(convertSchedule);
}

export async function saveSubTask(
  content: string,
  fatherTaskId: number,
  description?: string
): Promise<number> {
  return invoke<number>('db_save_sub_task', { content, fatherTaskId, description });
}

export async function searchSchedules(keyword: string): Promise<Schedule[]> {
  const result = await invoke<BackendSchedule[]>('db_search_schedules', { keyword });
  return result.map(convertSchedule);
}

// ========== MainTask API ==========

export async function loadMainTasks(): Promise<MainTask[]> {
  const result = await invoke<BackendMainTask[]>('db_load_main_tasks');
  return result.map(convertMainTask);
}

export async function searchMainTasks(keyword: string): Promise<MainTask[]> {
  const result = await invoke<BackendMainTask[]>('db_search_main_tasks', { keyword });
  return result.map(convertMainTask);
}

export async function saveMainTask(
  content: string,
  description?: string,
  priority: number = 0
): Promise<number> {
  return invoke<number>('db_save_main_task', { content, description, priority });
}

export async function updateMainTaskContent(id: number, content: string): Promise<void> {
  await invoke('db_update_main_task_content', { id, content });
}

export async function updateMainTaskDescription(id: number, description: string): Promise<void> {
  await invoke('db_update_main_task_description', { id, description: description || null });
}

export async function updateMainTaskCreateDate(id: number, createDate: string): Promise<void> {
  await invoke('db_update_main_task_create_date', { id, createDate });
}

export async function updateMainTaskDoneDate(id: number, doneDate: string | null): Promise<void> {
  await invoke('db_update_main_task_done_date', { id, doneDate });
}

export async function toggleMainTaskStatus(id: number, isDone: boolean): Promise<void> {
  await invoke('db_toggle_main_task_status', { id, isDone });
}

export async function updateMainTaskPriority(id: number, priority: number): Promise<void> {
  await invoke('db_update_main_task_priority', { id, priority });
}

export async function deleteMainTask(id: number): Promise<void> {
  await invoke('db_delete_main_task', { id });
}

// ========== Note API ==========

export async function loadNotes(): Promise<Note[]> {
  const result = await invoke<BackendNote[]>('db_load_notes');
  return result.map(convertNote);
}

export async function searchNotes(keyword: string): Promise<Note[]> {
  const result = await invoke<BackendNote[]>('db_search_notes', { keyword });
  return result.map(convertNote);
}

export async function getNote(id: number): Promise<Note | null> {
  const result = await invoke<BackendNote | null>('db_get_note', { id });
  return result ? convertNote(result) : null;
}

export async function createNote(title: string = '', content: string = ''): Promise<number> {
  return invoke<number>('db_create_note', { title, content });
}

export async function updateNote(id: number, title: string, content: string): Promise<void> {
  await invoke('db_update_note', { id, title, content });
}

export async function updateNoteTitle(id: number, title: string): Promise<void> {
  await invoke('db_update_note_title', { id, title });
}

export async function updateNoteContent(id: number, content: string): Promise<void> {
  await invoke('db_update_note_content', { id, content });
}

export async function updateNoteCreateDate(id: number, createDate: string): Promise<void> {
  await invoke('db_update_note_create_date', { id, createDate });
}

export async function deleteNote(id: number): Promise<void> {
  await invoke('db_delete_note', { id });
}

// ========== Cell Color API ==========

export async function updateScheduleColor(date: string, color: string): Promise<void> {
  await invoke('db_update_cell_color', { date, color });
}

export async function getCellColor(date: string): Promise<string> {
  return invoke<string>('db_get_cell_color', { date });
}

export async function loadCellColorMap(startDate: string, endDate: string): Promise<Map<string, string>> {
  const result = await invoke<CellMetadata[]>('db_load_cell_colors', { startDate, endDate });
  return new Map(result.map(m => [m.date, m.cell_color || '']));
}

// ========== Backup API ==========

export async function exportAllData(): Promise<ExportData> {
  const data = await invoke<BackendBackupData>('db_export_all_data');
  return {
    schedules: data.schedules.map(convertSchedule),
    mainTasks: data.main_tasks.map(convertMainTask),
    notes: data.notes.map(convertNote),
    cellMetadata: data.cell_metadata.map(m => ({ date: m.date, cell_color: m.cell_color || '' })),
  };
}

export async function importAndMergeData(data: ExportData): Promise<ImportStats> {
  // 转换为后端格式 (camelCase -> snake_case)
  const backendData = {
    schedules: data.schedules.map(s => ({
      id: s.id || 0,
      create_date: s.create_date || null,
      content: s.content,
      is_done: s.is_done,
      priority: s.priority,
      done_date: s.done_date || null,
      description: s.description || null,
      father_task: s.father_task || null,
    })),
    main_tasks: data.mainTasks.map(t => ({
      id: t.id || 0,
      content: t.content,
      description: t.description || null,
      is_done: t.is_done,
      priority: t.priority,
      create_date: t.create_date,
      done_date: t.done_date || null,
    })),
    notes: data.notes.map(n => ({
      id: n.id || 0,
      title: n.title,
      content: n.content,
      create_date: n.create_date,
    })),
    cell_metadata: data.cellMetadata.map(m => ({
      date: m.date,
      cell_color: m.cell_color || null,
    })),
  };

  const stats = await invoke<BackendImportStats>('db_import_and_merge_data', { data: backendData });
  return convertImportStats(stats);
}

export async function clearAllTables(): Promise<void> {
  await invoke('db_clear_all_tables');
}

export async function resetAutoIncrement(): Promise<void> {
  await invoke('db_reset_auto_increment');
}

// ========== Legacy Compatibility ==========

export async function loadAllSchedules(): Promise<{ schedules: Schedule[], cellColors: Map<string, string> }> {
  // 加载所有日程
  const today = new Date();
  const startDate = `${today.getFullYear()}-01-01`;
  const endDate = `${today.getFullYear() + 1}-12-31`;

  const schedules = await loadSchedules(startDate, endDate);
  const cellColors = await loadCellColorMap(startDate, endDate);

  return { schedules, cellColors };
}

export async function importSchedules(
  schedules: Schedule[],
  merge: boolean
): Promise<{ inserted: number, updated: number }> {
  // 简化实现：直接插入
  let inserted = 0;
  let updated = 0;

  if (merge) {
    // TODO: 实现合并逻辑
    for (const s of schedules) {
      try {
        await saveSchedule(s.create_date || '', s.content, s.is_done, s.done_date || undefined, s.description || undefined);
        inserted++;
      } catch {
        updated++;
      }
    }
  } else {
    for (const s of schedules) {
      await saveSchedule(s.create_date || '', s.content, s.is_done, s.done_date || undefined, s.description || undefined);
      inserted++;
    }
  }

  return { inserted, updated };
}

export async function importCellColors(
  colors: { date: string, color: string }[]
): Promise<{ inserted: number, updated: number }> {
  let inserted = 0;
  let updated = 0;

  for (const { date, color } of colors) {
    const existing = await getCellColor(date);
    if (existing) {
      updated++;
    } else {
      inserted++;
    }
    await updateScheduleColor(date, color);
  }

  return { inserted, updated };
}

export async function clearAllData(): Promise<void> {
  await invoke('db_clear_all_tables');
}
