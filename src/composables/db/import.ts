import type { Schedule } from '../../types';
import { getDatabase } from './connection';

/**
 * 更新单元格颜色
 */
export async function updateScheduleColor(date: string, color: string): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute(`
      INSERT INTO cell_metadata (date, cell_color) VALUES ($1, $2)
      ON CONFLICT(date) DO UPDATE SET cell_color = $2
    `, [date, color]);
  } catch (error) {
    console.error('Failed to update schedule color:', error);
    throw error;
  }
}

/**
 * 获取单元格颜色
 */
export async function getCellColor(date: string): Promise<string> {
  const db = getDatabase();
  if (!db) return '';

  try {
    const result = await db.select<{cell_color: string}[]>(
      'SELECT cell_color FROM cell_metadata WHERE date = $1',
      [date]
    );
    return result[0]?.cell_color || '';
  } catch (error) {
    console.error('Failed to get cell color:', error);
    return '';
  }
}

/**
 * 加载单元格颜色映射
 */
export async function loadCellColorMap(startDate: string, endDate: string): Promise<Map<string, string>> {
  const db = getDatabase();
  if (!db) return new Map();

  try {
    const metadata = await db.select<{date: string; cell_color: string}[]>(`
      SELECT * FROM cell_metadata
      WHERE date >= $1 AND date <= $2
    `, [startDate, endDate]);

    return new Map(metadata.map(m => [m.date, m.cell_color]));
  } catch (error) {
    console.error('Failed to load cell colors:', error);
    return new Map();
  }
}

/**
 * 加载所有日程和颜色数据（用于导出）
 */
export async function loadAllSchedules(): Promise<{ schedules: Schedule[], cellColors: Map<string, string> }> {
  const db = getDatabase();
  if (!db) return { schedules: [], cellColors: new Map() };

  try {
    const schedules = await db.select<Schedule[]>(
      'SELECT * FROM schedules ORDER BY create_date, id'
    );

    const metadata = await db.select<{date: string; cell_color: string}[]>(
      "SELECT date, cell_color FROM cell_metadata WHERE cell_color != ''"
    );

    const cellColors = new Map(metadata.map(m => [m.date, m.cell_color]));

    return { schedules, cellColors };
  } catch (error) {
    console.error('Failed to load all schedules:', error);
    return { schedules: [], cellColors: new Map() };
  }
}

/**
 * 导入日程数据
 */
export async function importSchedules(schedules: Schedule[], merge: boolean = false): Promise<{ inserted: number, updated: number }> {
  const db = getDatabase();
  if (!db) return { inserted: 0, updated: 0 };

  let inserted = 0;
  let updated = 0;

  try {
    if (merge) {
      for (const schedule of schedules) {
        const existing = await db.select<Schedule[]>(
          'SELECT * FROM schedules WHERE create_date = $1 AND content = $2',
          [schedule.create_date, schedule.content]
        );

        if (existing.length > 0) {
          await db.execute(
            'UPDATE schedules SET is_done = $1, priority = $2, done_date = $3, description = $4 WHERE id = $5',
            [schedule.is_done ? 1 : 0, schedule.priority, schedule.done_date || null, schedule.description || null, existing[0].id]
          );
          updated++;
        } else {
          await db.execute(
            'INSERT INTO schedules (create_date, content, is_done, priority, done_date, description) VALUES ($1, $2, $3, $4, $5, $6)',
            [schedule.create_date, schedule.content, schedule.is_done ? 1 : 0, schedule.priority, schedule.done_date || null, schedule.description || null]
          );
          inserted++;
        }
      }
    } else {
      for (const schedule of schedules) {
        await db.execute(
          'INSERT INTO schedules (create_date, content, is_done, priority, done_date, description) VALUES ($1, $2, $3, $4, $5, $6)',
          [schedule.create_date, schedule.content, schedule.is_done ? 1 : 0, schedule.priority, schedule.done_date || null, schedule.description || null]
        );
        inserted++;
      }
    }

    return { inserted, updated };
  } catch (error) {
    console.error('Failed to import schedules:', error);
    throw error;
  }
}

/**
 * 导入单元格颜色
 */
export async function importCellColors(cellColors: { date: string, color: string }[]): Promise<{ inserted: number, updated: number }> {
  const db = getDatabase();
  if (!db) return { inserted: 0, updated: 0 };

  let inserted = 0;
  let updated = 0;

  try {
    for (const { date, color } of cellColors) {
      const existing = await db.select<{ count: number }[]>(
        'SELECT COUNT(*) as count FROM cell_metadata WHERE date = $1',
        [date]
      );

      if (existing[0]?.count > 0) {
        updated++;
      } else {
        inserted++;
      }

      await db.execute(
        'INSERT INTO cell_metadata (date, cell_color) VALUES ($1, $2) ON CONFLICT(date) DO UPDATE SET cell_color = $2',
        [date, color]
      );
    }

    return { inserted, updated };
  } catch (error) {
    console.error('Failed to import cell colors:', error);
    throw error;
  }
}

/**
 * 清空所有数据
 */
export async function clearAllData(): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute('DELETE FROM schedules');
    await db.execute('DELETE FROM cell_metadata');
  } catch (error) {
    console.error('Failed to clear all data:', error);
    throw error;
  }
}
