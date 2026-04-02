/**
 * 文件导入导出工具
 * 支持 .db 备份和 .zip 压缩包导出
 */
import { save, open } from '@tauri-apps/plugin-dialog';
import { writeTextFile, writeFile, readFile, readTextFile } from '@tauri-apps/plugin-fs';
import JSZip from 'jszip';
import { decodeCSVBuffer } from './csv/encoding';
import { schedulesToCSV, csvToSchedules } from './csv/generator';
import type { Schedule } from '../types';
import type { MainTask } from '../composables/db/mainTask';
import type { Note } from '../composables/db/note';

/**
 * 备份数据结构（用于 CSV 导出）
 */
export interface ExportData {
  schedules: Schedule[];
  mainTasks: MainTask[];
  notes: Note[];
  cellMetadata: { date: string; cell_color: string }[];
}

/**
 * 将数据转换为 CSV 格式
 */
function dataToCSV(data: ExportData): Map<string, string> {
  const files = new Map<string, string>();

  // schedules.csv
  const scheduleHeader = 'id,create_date,content,is_done,priority,done_date,description,father_task';
  const scheduleLines = data.schedules.map(s => [
    s.id ?? '',
    s.create_date ?? '',
    escapeCSVField(s.content),
    s.is_done ? 'true' : 'false',
    s.priority ?? 0,
    s.done_date ?? '',
    escapeCSVField(s.description ?? ''),
    s.father_task ?? ''
  ].join(','));
  files.set('schedules.csv', [scheduleHeader, ...scheduleLines].join('\n'));

  // main_tasks.csv
  const taskHeader = 'id,content,description,is_done,priority,create_date,done_date';
  const taskLines = data.mainTasks.map(t => [
    t.id ?? '',
    escapeCSVField(t.content),
    escapeCSVField(t.description ?? ''),
    t.is_done ? 'true' : 'false',
    t.priority ?? 0,
    t.create_date,
    t.done_date ?? ''
  ].join(','));
  files.set('main_tasks.csv', [taskHeader, ...taskLines].join('\n'));

  // notes.csv
  const noteHeader = 'id,title,content,create_date';
  const noteLines = data.notes.map(n => [
    n.id ?? '',
    escapeCSVField(n.title),
    escapeCSVField(n.content ?? ''),
    n.create_date
  ].join(','));
  files.set('notes.csv', [noteHeader, ...noteLines].join('\n'));

  // cell_metadata.csv
  const metaHeader = 'date,cell_color';
  const metaLines = data.cellMetadata.map(m => [m.date, m.cell_color].join(','));
  files.set('cell_metadata.csv', [metaHeader, ...metaLines].join('\n'));

  return files;
}

/**
 * CSV 字段转义
 */
function escapeCSVField(field: string): string {
  if (field.includes(',') || field.includes('"') || field.includes('\n') || field.includes('\r')) {
    return '"' + field.replace(/"/g, '""') + '"';
  }
  return field;
}

/**
 * 导出为 zip 压缩包（包含所有表的 CSV）
 */
export async function exportAsZip(data: ExportData): Promise<boolean> {
  try {
    const filePath = await save({
      defaultPath: `chronos-backup-${getDateString()}.zip`,
      filters: [{ name: 'ZIP', extensions: ['zip'] }]
    });

    if (!filePath) return false;

    const zip = new JSZip();
    const csvFiles = dataToCSV(data);

    csvFiles.forEach((content, filename) => {
      zip.file(filename, content);
    });

    const zipBlob = await zip.generateAsync({ type: 'uint8array' });
    await writeFile(filePath, zipBlob);

    return true;
  } catch (error) {
    console.error('Failed to export zip:', error);
    throw new Error('导出压缩包失败，请重试');
  }
}

/**
 * 导出数据库备份文件（直接复制 .db 文件）
 * 注意：Tauri 插件限制，无法直接访问数据库文件
 * 改为导出 JSON 格式的完整数据
 */
export async function exportDatabaseBackup(data: ExportData): Promise<boolean> {
  try {
    const filePath = await save({
      defaultPath: `chronos-backup-${getDateString()}.json`,
      filters: [{ name: 'JSON', extensions: ['json'] }]
    });

    if (!filePath) return false;

    const jsonData = JSON.stringify(data, null, 2);
    await writeTextFile(filePath, jsonData);

    return true;
  } catch (error) {
    console.error('Failed to export backup:', error);
    throw new Error('导出备份失败，请重试');
  }
}

/**
 * 从 JSON 文件导入数据
 */
export async function importFromJson(): Promise<ExportData | null> {
  try {
    const filePath = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }]
    });

    if (!filePath) return null;

    const content = await readTextFile(filePath as string);
    const data = JSON.parse(content) as ExportData;

    // 验证数据结构
    if (!data.schedules || !data.mainTasks || !data.notes || !data.cellMetadata) {
      throw new Error('备份文件格式不正确');
    }

    return data;
  } catch (error) {
    console.error('Failed to import backup:', error);
    const message = error instanceof Error ? error.message : '导入备份失败，请重试';
    throw new Error(message);
  }
}

/**
 * 导入 CSV 压缩包
 */
export async function importFromZip(): Promise<ExportData | null> {
  try {
    const filePath = await open({
      multiple: false,
      filters: [{ name: 'ZIP', extensions: ['zip'] }]
    });

    if (!filePath) return null;

    const buffer = await readFile(filePath as string);
    const zip = await JSZip.loadAsync(buffer);

    const data: ExportData = {
      schedules: [],
      mainTasks: [],
      notes: [],
      cellMetadata: []
    };

    // 解析 schedules.csv
    const schedulesFile = zip.file('schedules.csv');
    if (schedulesFile) {
      const content = await schedulesFile.async('text');
      data.schedules = parseSchedulesCSV(content);
    }

    // 解析 main_tasks.csv
    const tasksFile = zip.file('main_tasks.csv');
    if (tasksFile) {
      const content = await tasksFile.async('text');
      data.mainTasks = parseMainTasksCSV(content);
    }

    // 解析 notes.csv
    const notesFile = zip.file('notes.csv');
    if (notesFile) {
      const content = await notesFile.async('text');
      data.notes = parseNotesCSV(content);
    }

    // 解析 cell_metadata.csv
    const metaFile = zip.file('cell_metadata.csv');
    if (metaFile) {
      const content = await metaFile.async('text');
      data.cellMetadata = parseCellMetadataCSV(content);
    }

    return data;
  } catch (error) {
    console.error('Failed to import zip:', error);
    const message = error instanceof Error ? error.message : '导入压缩包失败，请重试';
    throw new Error(message);
  }
}

/**
 * 解析 schedules CSV
 */
function parseSchedulesCSV(content: string): Schedule[] {
  const lines = content.split('\n').filter(l => l.trim());
  if (lines.length < 2) return [];

  const schedules: Schedule[] = [];
  for (let i = 1; i < lines.length; i++) {
    const fields = parseCSVLine(lines[i]);
    if (fields.length < 8) continue;

    schedules.push({
      id: fields[0] ? parseInt(fields[0], 10) : undefined,
      create_date: fields[1],
      content: fields[2],
      is_done: fields[3].toLowerCase() === 'true',
      priority: parseInt(fields[4], 10) || 0,
      done_date: fields[5] || undefined,
      description: fields[6] || undefined,
      father_task: fields[7] ? parseInt(fields[7], 10) : undefined
    });
  }
  return schedules;
}

/**
 * 解析 main_tasks CSV
 */
function parseMainTasksCSV(content: string): MainTask[] {
  const lines = content.split('\n').filter(l => l.trim());
  if (lines.length < 2) return [];

  const tasks: MainTask[] = [];
  for (let i = 1; i < lines.length; i++) {
    const fields = parseCSVLine(lines[i]);
    if (fields.length < 7) continue;

    tasks.push({
      id: fields[0] ? parseInt(fields[0], 10) : undefined,
      content: fields[1],
      description: fields[2] || undefined,
      is_done: fields[3].toLowerCase() === 'true',
      priority: parseInt(fields[4], 10) || 0,
      create_date: fields[5],
      done_date: fields[6] || undefined
    });
  }
  return tasks;
}

/**
 * 解析 notes CSV
 */
function parseNotesCSV(content: string): Note[] {
  const lines = content.split('\n').filter(l => l.trim());
  if (lines.length < 2) return [];

  const notes: Note[] = [];
  for (let i = 1; i < lines.length; i++) {
    const fields = parseCSVLine(lines[i]);
    if (fields.length < 4) continue;

    notes.push({
      id: fields[0] ? parseInt(fields[0], 10) : undefined,
      title: fields[1],
      content: fields[2],
      create_date: fields[3]
    });
  }
  return notes;
}

/**
 * 解析 cell_metadata CSV
 */
function parseCellMetadataCSV(content: string): { date: string; cell_color: string }[] {
  const lines = content.split('\n').filter(l => l.trim());
  if (lines.length < 2) return [];

  const metadata: { date: string; cell_color: string }[] = [];
  for (let i = 1; i < lines.length; i++) {
    const fields = parseCSVLine(lines[i]);
    if (fields.length < 2) continue;

    if (fields[1]) {
      metadata.push({ date: fields[0], cell_color: fields[1] });
    }
  }
  return metadata;
}

/**
 * 解析 CSV 行（处理引号转义）
 */
function parseCSVLine(line: string): string[] {
  const result: string[] = [];
  let current = '';
  let inQuotes = false;

  for (let i = 0; i < line.length; i++) {
    const char = line[i];
    const nextChar = line[i + 1];

    if (inQuotes) {
      if (char === '"' && nextChar === '"') {
        current += '"';
        i++;
      } else if (char === '"') {
        inQuotes = false;
      } else {
        current += char;
      }
    } else {
      if (char === '"') {
        inQuotes = true;
      } else if (char === ',') {
        result.push(current);
        current = '';
      } else {
        current += char;
      }
    }
  }
  result.push(current);
  return result;
}

/**
 * 获取日期字符串
 */
function getDateString(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
}

/**
 * Export data to CSV file using Tauri dialog
 */
export async function exportToFile(data: string, filename: string): Promise<boolean> {
  try {
    const filePath = await save({
      defaultPath: filename,
      filters: [{ name: 'CSV', extensions: ['csv'] }]
    });

    if (!filePath) return false;

    await writeTextFile(filePath, data);
    return true;
  } catch (error) {
    console.error('Failed to export file:', error);
    throw new Error('导出文件失败，请重试');
  }
}

/**
 * Import data from CSV file using Tauri dialog
 */
export async function importFromFile(): Promise<string | null> {
  try {
    const filePath = await open({
      multiple: false,
      filters: [{ name: 'CSV', extensions: ['csv'] }]
    });

    if (!filePath) return null;

    const buffer = await readFile(filePath as string);
    const content = decodeCSVBuffer(buffer);
    return content;
  } catch (error) {
    console.error('Failed to import file:', error);
    const message = error instanceof Error ? error.message : '导入文件失败，请重试';
    throw new Error(message);
  }
}

export { schedulesToCSV, csvToSchedules };

