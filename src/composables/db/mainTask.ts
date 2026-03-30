import { getDatabase } from './connection';

/**
 * 主任务类型
 */
export interface MainTask {
  id?: number;
  content: string;
  description?: string;
  is_done: boolean;
  priority: number;
  create_date: string;
  done_date?: string;
}

/**
 * 加载所有主任务
 */
export async function loadMainTasks(): Promise<MainTask[]> {
  const db = getDatabase();
  if (!db) return [];

  try {
    const tasks = await db.select<MainTask[]>(`
      SELECT * FROM main_tasks
      ORDER BY
        is_done ASC,
        CASE WHEN is_done = 0 THEN create_date ELSE COALESCE(done_date, create_date) END DESC,
        id DESC
    `);
    return tasks;
  } catch (error) {
    console.error('Failed to load main tasks:', error);
    return [];
  }
}

/**
 * 保存主任务
 */
export async function saveMainTask(
  content: string,
  description?: string,
  priority: number = 0
): Promise<number | null> {
  const db = getDatabase();
  if (!db) return null;

  try {
    const now = new Date();
    const createDate = now.getFullYear() + '-' +
      String(now.getMonth() + 1).padStart(2, '0') + '-' +
      String(now.getDate()).padStart(2, '0');

    const result = await db.execute(
      'INSERT INTO main_tasks (content, description, is_done, priority, create_date) VALUES ($1, $2, 0, $3, $4)',
      [content.trim(), description?.trim() || null, priority, createDate]
    );

    return result.lastInsertId ?? null;
  } catch (error) {
    console.error('Failed to save main task:', error);
    throw error;
  }
}

/**
 * 更新主任务内容
 */
export async function updateMainTaskContent(taskId: number, content: string): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute(
      'UPDATE main_tasks SET content = $1 WHERE id = $2',
      [content.trim(), taskId]
    );
  } catch (error) {
    console.error('Failed to update main task content:', error);
    throw error;
  }
}

/**
 * 更新主任务描述
 */
export async function updateMainTaskDescription(taskId: number, description: string): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute(
      'UPDATE main_tasks SET description = $1 WHERE id = $2',
      [description.trim() || null, taskId]
    );
  } catch (error) {
    console.error('Failed to update main task description:', error);
    throw error;
  }
}

/**
 * 切换主任务完成状态
 */
export async function toggleMainTaskStatus(taskId: number, isDone: boolean): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    if (isDone) {
      const now = new Date();
      const doneDate = now.getFullYear() + '-' +
        String(now.getMonth() + 1).padStart(2, '0') + '-' +
        String(now.getDate()).padStart(2, '0');
      await db.execute(
        'UPDATE main_tasks SET is_done = 1, done_date = $1 WHERE id = $2',
        [doneDate, taskId]
      );
    } else {
      await db.execute(
        'UPDATE main_tasks SET is_done = 0, done_date = NULL WHERE id = $1',
        [taskId]
      );
    }
  } catch (error) {
    console.error('Failed to toggle main task status:', error);
    throw error;
  }
}

/**
 * 更新主任务优先级
 */
export async function updateMainTaskPriority(taskId: number, priority: number): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute(
      'UPDATE main_tasks SET priority = $1 WHERE id = $2',
      [priority, taskId]
    );
  } catch (error) {
    console.error('Failed to update main task priority:', error);
    throw error;
  }
}

/**
 * 删除主任务
 */
export async function deleteMainTask(taskId: number): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute('DELETE FROM main_tasks WHERE id = $1', [taskId]);
  } catch (error) {
    console.error('Failed to delete main task:', error);
    throw error;
  }
}

/**
 * 搜索主任务
 */
export async function searchMainTasks(keyword: string): Promise<MainTask[]> {
  const db = getDatabase();
  if (!db) return [];

  try {
    const tasks = await db.select<MainTask[]>(`
      SELECT * FROM main_tasks
      WHERE content LIKE $1 OR description LIKE $1
      ORDER BY
        is_done ASC,
        CASE WHEN is_done = 0 THEN create_date ELSE COALESCE(done_date, create_date) END DESC,
        id DESC
    `, [`%${keyword.trim()}%`]);
    return tasks;
  } catch (error) {
    console.error('Failed to search main tasks:', error);
    return [];
  }
}

/**
 * 更新主任务创建日期
 */
export async function updateMainTaskCreateDate(taskId: number, createDate: string): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute(
      'UPDATE main_tasks SET create_date = $1 WHERE id = $2',
      [createDate, taskId]
    );
  } catch (error) {
    console.error('Failed to update main task create date:', error);
    throw error;
  }
}

/**
 * 更新主任务完成日期
 */
export async function updateMainTaskDoneDate(taskId: number, doneDate: string | null): Promise<void> {
  const db = getDatabase();
  if (!db) return;

  try {
    await db.execute(
      'UPDATE main_tasks SET done_date = $1 WHERE id = $2',
      [doneDate || null, taskId]
    );
  } catch (error) {
    console.error('Failed to update main task done date:', error);
    throw error;
  }
}
