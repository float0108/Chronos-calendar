/**
 * 数据库备份和恢复功能
 * 支持导出 .db 文件和导入合并
 */
import { getDatabase } from './connection';
import type { Schedule } from '../../types';
import type { MainTask } from './mainTask';
import type { Note } from './note';

/**
 * 数据库完整数据结构
 */
export interface BackupData {
  schedules: Schedule[];
  mainTasks: MainTask[];
  notes: Note[];
  cellMetadata: { date: string; cell_color: string }[];
}

/**
 * 导出所有数据
 */
export async function exportAllData(): Promise<BackupData> {
  const db = getDatabase();
  if (!db) throw new Error('数据库未初始化');

  const schedules = await db.select<Schedule[]>('SELECT * FROM schedules ORDER BY id');
  const mainTasks = await db.select<MainTask[]>('SELECT * FROM main_tasks ORDER BY id');
  const notes = await db.select<Note[]>('SELECT * FROM notes ORDER BY id');
  const cellMetadata = await db.select<{ date: string; cell_color: string }[]>(
    "SELECT date, cell_color FROM cell_metadata WHERE cell_color != ''"
  );

  return { schedules, mainTasks, notes, cellMetadata };
}

/**
 * 导入合并数据
 * 规则：同 ID 更新，新 ID 补录
 */
export async function importAndMergeData(data: BackupData): Promise<{
  schedules: { inserted: number; updated: number };
  mainTasks: { inserted: number; updated: number };
  notes: { inserted: number; updated: number };
  cellMetadata: { inserted: number; updated: number };
}> {
  const db = getDatabase();
  if (!db) throw new Error('数据库未初始化');

  const stats = {
    schedules: { inserted: 0, updated: 0 },
    mainTasks: { inserted: 0, updated: 0 },
    notes: { inserted: 0, updated: 0 },
    cellMetadata: { inserted: 0, updated: 0 }
  };

  // 导入 schedules
  for (const schedule of data.schedules) {
    if (schedule.id) {
      const existing = await db.select<{ count: number }[]>(
        'SELECT COUNT(*) as count FROM schedules WHERE id = $1',
        [schedule.id]
      );
      if (existing[0]?.count > 0) {
        await db.execute(
          `UPDATE schedules SET create_date = $1, content = $2, is_done = $3, priority = $4, done_date = $5, description = $6, father_task = $7 WHERE id = $8`,
          [
            schedule.create_date,
            schedule.content,
            schedule.is_done ? 1 : 0,
            schedule.priority ?? 0,
            schedule.done_date || null,
            schedule.description || null,
            schedule.father_task || null,
            schedule.id
          ]
        );
        stats.schedules.updated++;
      } else {
        await db.execute(
          `INSERT INTO schedules (id, create_date, content, is_done, priority, done_date, description, father_task) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`,
          [
            schedule.id,
            schedule.create_date,
            schedule.content,
            schedule.is_done ? 1 : 0,
            schedule.priority ?? 0,
            schedule.done_date || null,
            schedule.description || null,
            schedule.father_task || null
          ]
        );
        stats.schedules.inserted++;
      }
    }
  }

  // 导入 main_tasks
  for (const task of data.mainTasks) {
    if (task.id) {
      const existing = await db.select<{ count: number }[]>(
        'SELECT COUNT(*) as count FROM main_tasks WHERE id = $1',
        [task.id]
      );
      if (existing[0]?.count > 0) {
        await db.execute(
          `UPDATE main_tasks SET content = $1, description = $2, is_done = $3, priority = $4, create_date = $5, done_date = $6 WHERE id = $7`,
          [
            task.content,
            task.description || null,
            task.is_done ? 1 : 0,
            task.priority ?? 0,
            task.create_date,
            task.done_date || null,
            task.id
          ]
        );
        stats.mainTasks.updated++;
      } else {
        await db.execute(
          `INSERT INTO main_tasks (id, content, description, is_done, priority, create_date, done_date) VALUES ($1, $2, $3, $4, $5, $6, $7)`,
          [
            task.id,
            task.content,
            task.description || null,
            task.is_done ? 1 : 0,
            task.priority ?? 0,
            task.create_date,
            task.done_date || null
          ]
        );
        stats.mainTasks.inserted++;
      }
    }
  }

  // 导入 notes
  for (const note of data.notes) {
    if (note.id) {
      const existing = await db.select<{ count: number }[]>(
        'SELECT COUNT(*) as count FROM notes WHERE id = $1',
        [note.id]
      );
      if (existing[0]?.count > 0) {
        await db.execute(
          `UPDATE notes SET title = $1, content = $2, create_date = $3 WHERE id = $4`,
          [note.title, note.content, note.create_date, note.id]
        );
        stats.notes.updated++;
      } else {
        await db.execute(
          `INSERT INTO notes (id, title, content, create_date) VALUES ($1, $2, $3, $4)`,
          [note.id, note.title, note.content, note.create_date]
        );
        stats.notes.inserted++;
      }
    }
  }

  // 导入 cell_metadata (使用 UPSERT)
  for (const meta of data.cellMetadata) {
    const existing = await db.select<{ count: number }[]>(
      'SELECT COUNT(*) as count FROM cell_metadata WHERE date = $1',
      [meta.date]
    );
    if (existing[0]?.count > 0) {
      stats.cellMetadata.updated++;
    } else {
      stats.cellMetadata.inserted++;
    }
    await db.execute(
      `INSERT INTO cell_metadata (date, cell_color) VALUES ($1, $2) ON CONFLICT(date) DO UPDATE SET cell_color = $2`,
      [meta.date, meta.cell_color]
    );
  }

  return stats;
}

/**
 * 清空所有数据（用于覆盖导入前）
 */
export async function clearAllTables(): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  await db.execute('DELETE FROM schedules');
  await db.execute('DELETE FROM main_tasks');
  await db.execute('DELETE FROM notes');
  await db.execute('DELETE FROM cell_metadata');
}

/**
 * 重置自增 ID 序列
 * SQLite 在删除数据后 ID 不会重置，需要手动处理
 */
export async function resetAutoIncrement(): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  // SQLite 使用 DELETE FROM 后，需要更新 sqlite_sequence
  await db.execute("DELETE FROM sqlite_sequence WHERE name IN ('schedules', 'main_tasks', 'notes')");
}
