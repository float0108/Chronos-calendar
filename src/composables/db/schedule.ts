import type { Schedule } from '../../types';
import { getDatabase } from './connection';

/**
 * 辅助函数：加载单元格颜色映射
 */
async function loadCellColorMap(startDate: string, endDate: string): Promise<Map<string, string>> {
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
 * 辅助函数：为日程附加颜色并添加只有颜色的空记录
 */
function attachColorsToSchedules(
  schedules: Schedule[],
  colorMap: Map<string, string>,
  dateField: 'create_date' | 'done_date' = 'create_date'
): Schedule[] {
  // 按日期分组
  const grouped = new Map<string, Schedule[]>();
  schedules.forEach(s => {
    const date = dateField === 'done_date' ? (s.done_date || s.create_date) : s.create_date;
    if (!grouped.has(date)) grouped.set(date, []);
    grouped.get(date)!.push(s);
  });

  const result: Schedule[] = [];

  // 为每个日期的记录附加颜色
  grouped.forEach((items, date) => {
    const color = colorMap.get(date);
    if (color && items.length > 0) {
      items[0].cell_color = color;
    }

    // 如果使用 done_date，需要将其映射到 create_date
    if (dateField === 'done_date') {
      items.forEach(item => {
        if (item.done_date) {
          item.create_date = item.done_date;
        }
      });
    }

    result.push(...items);
  });

  // 添加只有颜色没有内容的日期（用于显示背景色）
  colorMap.forEach((color, date) => {
    if (!grouped.has(date)) {
      result.push({
        id: -1,
        create_date: date,
        content: '',
        is_done: dateField === 'done_date',
        priority: 0,
        cell_color: color,
        ...(dateField === 'done_date' && { done_date: date })
      });
    }
  });

  return result;
}

/**
 * 加载日程（按创建日期）
 */
export async function loadSchedules(startDate: string, endDate: string): Promise<Schedule[]> {
  const db = getDatabase();
  if (!db) return [];

  try {
    const schedules = await db.select<Schedule[]>(`
      SELECT * FROM schedules
      WHERE create_date >= $1 AND create_date <= $2
      ORDER BY id ASC
    `, [startDate, endDate]);

    const colorMap = await loadCellColorMap(startDate, endDate);
    return attachColorsToSchedules(schedules, colorMap);
  } catch (error) {
    console.error('Failed to load schedules:', error);
    return [];
  }
}

/**
 * 加载待办日程
 */
export async function loadTodoSchedules(startDate: string, endDate: string): Promise<Schedule[]> {
  const db = getDatabase();
  if (!db) return [];

  try {
    const schedules = await db.select<Schedule[]>(`
      SELECT * FROM schedules
      WHERE create_date >= $1 AND create_date <= $2
      ORDER BY create_date ASC, is_done ASC, id ASC
    `, [startDate, endDate]);

    const colorMap = await loadCellColorMap(startDate, endDate);
    return attachColorsToSchedules(schedules, colorMap);
  } catch (error) {
    console.error('Failed to load todo schedules:', error);
    return [];
  }
}

/**
 * 加载已完成日程（按完成日期）
 */
export async function loadDoneSchedules(startDate: string, endDate: string): Promise<Schedule[]> {
  const db = getDatabase();
  if (!db) return [];

  try {
    const schedules = await db.select<Schedule[]>(`
      SELECT * FROM schedules
      WHERE done_date >= $1 AND done_date <= $2 AND is_done = 1
      ORDER BY done_date, id ASC
    `, [startDate, endDate]);

    const colorMap = await loadCellColorMap(startDate, endDate);
    return attachColorsToSchedules(schedules, colorMap, 'done_date');
  } catch (error) {
    console.error('Failed to load done schedules:', error);
    return [];
  }
}

/**
 * 保存日程
 */
export async function saveSchedule(
  createDate: string,
  content: string,
  isDone: boolean = false,
  doneDate?: string,
  description?: string,
  fatherTask?: number
): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute(
      'INSERT INTO schedules (create_date, content, is_done, priority, done_date, description, father_task) VALUES ($1, $2, $3, $4, $5, $6, $7)',
      [createDate, content.trim(), isDone ? 1 : 0, 0, doneDate || null, description || null, fatherTask || null]
    );
  } catch (error) {
    console.error('Failed to save schedule:', error);
    throw error;
  }
}

/**
 * 删除日程
 */
export async function deleteSchedule(scheduleId: number): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute('DELETE FROM schedules WHERE id = $1', [scheduleId]);
  } catch (error) {
    console.error('Failed to delete schedule:', error);
    throw error;
  }
}

/**
 * 删除某日期的所有日程
 */
export async function deleteSchedulesByDate(date: string): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute('DELETE FROM schedules WHERE create_date = $1', [date]);
  } catch (error) {
    console.error('Failed to delete schedules:', error);
    throw error;
  }
}

/**
 * 切换日程完成状态
 */
export async function toggleScheduleStatus(scheduleId: number, isDone: boolean): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    if (isDone) {
      const now = new Date();
      const doneDate = now.getFullYear() + '-' +
        String(now.getMonth() + 1).padStart(2, '0') + '-' +
        String(now.getDate()).padStart(2, '0');
      await db.execute(
        'UPDATE schedules SET is_done = 1, done_date = $1 WHERE id = $2',
        [doneDate, scheduleId]
      );
    } else {
      await db.execute(
        'UPDATE schedules SET is_done = 0, done_date = NULL WHERE id = $1',
        [scheduleId]
      );
    }
  } catch (error) {
    console.error('Failed to toggle schedule:', error);
    throw error;
  }
}

/**
 * 更新日程描述
 */
export async function updateScheduleDescription(scheduleId: number, description: string): Promise<void> {
  const db = getDatabase();
  if (!db) return;
  await db.execute(
    'UPDATE schedules SET description = $1 WHERE id = $2',
    [description.trim() || null, scheduleId]
  );
}

/**
 * 更新日程内容
 */
export async function updateScheduleContent(scheduleId: number, content: string): Promise<void> {
  const db = getDatabase();
  if (!db) return;
  await db.execute(
    'UPDATE schedules SET content = $1 WHERE id = $2',
    [content.trim(), scheduleId]
  );
}

/**
 * 更新日程日期
 * @param scheduleId 日程 ID
 * @param field 要更新的字段: 'create_date' 或 'done_date'
 * @param date 新日期值 (YYYY-MM-DD 格式)
 */
export async function updateScheduleDate(scheduleId: number, field: 'create_date' | 'done_date', date: string): Promise<void> {
  const db = getDatabase();
  if (!db) return;
  await db.execute(
    `UPDATE schedules SET ${field} = $1 WHERE id = $2`,
    [date, scheduleId]
  );
}

/**
 * 更新日程关联的主任务
 */
export async function updateScheduleFatherTask(scheduleId: number, fatherTask: number | null): Promise<void> {
  const db = getDatabase();
  if (!db) return;
  await db.execute(
    'UPDATE schedules SET father_task = $1 WHERE id = $2',
    [fatherTask, scheduleId]
  );
}

/**
 * 加载关联到某个主任务的所有日程
 * 排序：未完成在前，内部按创建时间升序（越早在上面）
 */
export async function loadSchedulesByFatherTask(fatherTaskId: number): Promise<Schedule[]> {
  const db = getDatabase();
  if (!db) return [];

  try {
    const schedules = await db.select<Schedule[]>(`
      SELECT * FROM schedules
      WHERE father_task = $1
      ORDER BY is_done ASC, create_date DESC, id DESC
    `, [fatherTaskId]);
    return schedules;
  } catch (error) {
    console.error('Failed to load schedules by father task:', error);
    return [];
  }
}

/**
 * 保存子任务（日程，create_date 可为空）
 */
export async function saveSubTask(
  content: string,
  fatherTaskId: number,
  description?: string
): Promise<number | null> {
  const db = getDatabase();
  if (!db) return null;

  try {
    const result = await db.execute(
      'INSERT INTO schedules (create_date, content, is_done, priority, description, father_task) VALUES (NULL, $1, 0, 0, $2, $3)',
      [content.trim(), description || null, fatherTaskId]
    );
    return result.lastInsertId ?? null;
  } catch (error) {
    console.error('Failed to save sub task:', error);
    throw error;
  }
}
