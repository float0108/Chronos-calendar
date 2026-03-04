import { ref } from 'vue';
import Database from '@tauri-apps/plugin-sql';
import type { Schedule } from '../types';

const db = ref<Database | null>(null);

export function useDatabase() {
  async function initDatabase(): Promise<void> {
    try {
      db.value = await Database.load('sqlite:chronos.db');

      // 检查是否需要迁移表结构
      let needsMigration = false;
      try {
        const columns = await db.value!.select<{name: string}[]>(
          "SELECT name FROM pragma_table_info('schedules')"
        );
        const columnNames = columns.map(c => c.name);

        // 检查是否存在旧列名
        const hasOldDateColumn = columnNames.includes('date');
        const hasCreatedAtColumn = columnNames.includes('created_at');
        const hasDoneTimeColumn = columnNames.includes('done_time');

        if (hasOldDateColumn || hasCreatedAtColumn || hasDoneTimeColumn) {
          needsMigration = true;
          console.log('Table structure migration needed...');
        }
      } catch (e: any) {
        // 表不存在，首次创建
        console.log('Creating new table');
      }

      if (needsMigration) {
        // 创建新表
        await db.value!.execute(`
          CREATE TABLE IF NOT EXISTS schedules_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            create_date TEXT NOT NULL,
            content TEXT NOT NULL,
            is_done INTEGER DEFAULT 0,
            priority INTEGER DEFAULT 0,
            done_date TEXT,
            description TEXT
          )
        `);

        // 迁移数据（从旧列映射到新列）
        await db.value!.execute(`
          INSERT INTO schedules_new (id, create_date, content, is_done, priority, done_date, description)
          SELECT
            id,
            date,
            content,
            is_done,
            priority,
            done_time,
            description
          FROM schedules
        `);

        // 删除旧表
        await db.value!.execute('DROP TABLE schedules');

        // 重命名新表
        await db.value!.execute('ALTER TABLE schedules_new RENAME TO schedules');

        console.log('Table structure migration completed');
      } else {
        // 创建新表（首次运行）
        await db.value!.execute(`
          CREATE TABLE IF NOT EXISTS schedules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            create_date TEXT NOT NULL,
            content TEXT NOT NULL,
            is_done INTEGER DEFAULT 0,
            priority INTEGER DEFAULT 0,
            done_date TEXT,
            description TEXT
          )
        `);
      }

      await db.value!.execute(`
        CREATE INDEX IF NOT EXISTS idx_schedules_create_date ON schedules(create_date)
      `);

      // 单元格元数据表 - 存储颜色等元数据（与内容分离）
      await db.value!.execute(`
        CREATE TABLE IF NOT EXISTS cell_metadata (
          date TEXT PRIMARY KEY,
          cell_color TEXT DEFAULT ''
        )
      `);
    } catch (error) {
      console.error('Database initialization failed:', error);
      throw error;
    }

    // 迁移：将旧表中的颜色数据迁移到新表（一次性）
    try {
      const hasOldColorColumn = await db.value!.select<{count: number}[]>(
        "SELECT COUNT(*) as count FROM pragma_table_info('schedules') WHERE name = 'cell_color'"
      );
      if (hasOldColorColumn[0]?.count > 0) {
        // 迁移旧数据
        await db.value!.execute(`
          INSERT OR REPLACE INTO cell_metadata (date, cell_color)
          SELECT create_date, cell_color FROM schedules
          WHERE cell_color != '' AND cell_color IS NOT NULL
          GROUP BY create_date
        `);
        console.log('Color data migrated to cell_metadata table');
      }
    } catch (e: any) {
      console.log('Migration check:', e?.message || 'ignored');
    }

    try {
      await db.value!.execute('ALTER TABLE schedules ADD COLUMN is_done INTEGER DEFAULT 0');
    } catch (e: any) {
      console.log('is_done column migration:', e?.message || 'ignored');
    }
  }

  async function loadSchedules(startDate: string, endDate: string): Promise<Schedule[]> {
    if (!db.value) return [];

    try {
      // 获取日程内容（使用日期范围）
      const schedules = await db.value.select<Schedule[]>(`
        SELECT * FROM schedules
        WHERE create_date >= $1 AND create_date <= $2
        ORDER BY id ASC
      `, [startDate, endDate]);

      // 获取颜色元数据
      const metadata = await db.value.select<{date: string; cell_color: string}[]>(`
        SELECT * FROM cell_metadata
        WHERE date >= $1 AND date <= $2
      `, [startDate, endDate]);

      // 创建颜色映射
      const colorMap = new Map(metadata.map(m => [m.date, m.cell_color]));

      // 按日期分组处理日程
      const grouped = new Map<string, Schedule[]>();
      schedules.forEach(s => {
        if (!grouped.has(s.create_date)) grouped.set(s.create_date, []);
        grouped.get(s.create_date)!.push(s);
      });

      const result: Schedule[] = [];

      // 为每个日期的记录附加颜色
      grouped.forEach((items, date) => {
        const color = colorMap.get(date);
        if (color && items.length > 0) {
          items[0].cell_color = color;
        }
        result.push(...items);
      });

      // 添加只有颜色没有内容的日期（用于显示背景色）
      colorMap.forEach((color, date) => {
        if (!grouped.has(date)) {
          result.push({
            id: -1, // 标记为虚拟记录
            create_date: date,
            content: '',
            is_done: false,
            priority: 0,
            cell_color: color
          });
        }
      });

      return result;
    } catch (error) {
      console.error('Failed to load schedules:', error);
      return [];
    }
  }

  async function loadTodoSchedules(startDate: string, endDate: string): Promise<Schedule[]> {
    if (!db.value) return [];

    try {
      // 加载所有日程（包括已完成和未完成），按日期和完成状态排序
      const schedules = await db.value.select<Schedule[]>(`
        SELECT * FROM schedules
        WHERE create_date >= $1 AND create_date <= $2
        ORDER BY create_date ASC, is_done ASC, id ASC
      `, [startDate, endDate]);

      // 获取颜色元数据
      const metadata = await db.value.select<{date: string; cell_color: string}[]>(`
        SELECT * FROM cell_metadata
        WHERE date >= $1 AND date <= $2
      `, [startDate, endDate]);

      // 创建颜色映射
      const colorMap = new Map(metadata.map(m => [m.date, m.cell_color]));

      // 按日期分组处理日程
      const grouped = new Map<string, Schedule[]>();
      schedules.forEach(s => {
        if (!grouped.has(s.create_date)) grouped.set(s.create_date, []);
        grouped.get(s.create_date)!.push(s);
      });

      const result: Schedule[] = [];

      // 为每个日期的记录附加颜色
      grouped.forEach((items, date) => {
        const color = colorMap.get(date);
        if (color && items.length > 0) {
          items[0].cell_color = color;
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
            is_done: false,
            priority: 0,
            cell_color: color
          });
        }
      });

      return result;
    } catch (error) {
      console.error('Failed to load todo schedules:', error);
      return [];
    }
  }

  async function loadDoneSchedules(startDate: string, endDate: string): Promise<Schedule[]> {
    if (!db.value) return [];

    try {
      // 加载已完成的日程（is_done = 1），按 done_date 分组
      const schedules = await db.value.select<Schedule[]>(`
        SELECT * FROM schedules
        WHERE done_date >= $1 AND done_date <= $2 AND is_done = 1
        ORDER BY done_date, id ASC
      `, [startDate, endDate]);

      // 获取颜色元数据
      const metadata = await db.value.select<{date: string; cell_color: string}[]>(`
        SELECT * FROM cell_metadata
        WHERE date >= $1 AND date <= $2
      `, [startDate, endDate]);

      // 创建颜色映射
      const colorMap = new Map(metadata.map(m => [m.date, m.cell_color]));

      // 按 done_date 分组处理日程
      const grouped = new Map<string, Schedule[]>();
      schedules.forEach(s => {
        const date = s.done_date || s.create_date; // 使用 done_date 作为分组依据
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
        // 将 done_date 映射到 create_date 以便前端统一处理
        items.forEach(item => {
          if (item.done_date) {
            item.create_date = item.done_date;
          }
        });
        result.push(...items);
      });

      // 添加只有颜色没有内容的日期（用于显示背景色）
      colorMap.forEach((color, date) => {
        if (!grouped.has(date)) {
          result.push({
            id: -1,
            create_date: date,
            content: '',
            is_done: true,
            priority: 0,
            cell_color: color,
            done_date: date
          });
        }
      });

      return result;
    } catch (error) {
      console.error('Failed to load done schedules:', error);
      return [];
    }
  }

  async function saveSchedule(createDate: string, content: string, isDone: boolean = false, doneDate?: string, description?: string): Promise<void> {
    if (!db.value) return;

    try {
      await db.value.execute(
        'INSERT INTO schedules (create_date, content, is_done, priority, done_date, description) VALUES ($1, $2, $3, $4, $5, $6)',
        [createDate, content.trim(), isDone ? 1 : 0, 0, doneDate || null, description || null]
      );
    } catch (error) {
      console.error('Failed to save schedule:', error);
      throw error;
    }
  }

  async function deleteSchedule(scheduleId: number): Promise<void> {
    if (!db.value) return;

    try {
      await db.value.execute('DELETE FROM schedules WHERE id = $1', [scheduleId]);
    } catch (error) {
      console.error('Failed to delete schedule:', error);
      throw error;
    }
  }

  async function deleteSchedulesByDate(date: string): Promise<void> {
    if (!db.value) return;

    try {
      // 删除该日期的所有日程内容（颜色保留在 cell_metadata 表中）
      await db.value.execute('DELETE FROM schedules WHERE create_date = $1', [date]);
    } catch (error) {
      console.error('Failed to delete schedules:', error);
      throw error;
    }
  }

  async function toggleScheduleStatus(scheduleId: number, isDone: boolean): Promise<void> {
    if (!db.value) return;

    try {
      if (isDone) {
        // 完成时：设置 is_done=1, done_date=当前日期
        const now = new Date();
        const doneDate = now.getFullYear() + '-' +
          String(now.getMonth() + 1).padStart(2, '0') + '-' +
          String(now.getDate()).padStart(2, '0');
        await db.value.execute(
          'UPDATE schedules SET is_done = 1, done_date = $1 WHERE id = $2',
          [doneDate, scheduleId]
        );
      } else {
        // 取消完成：设置 is_done=0, done_date=NULL
        await db.value.execute(
          'UPDATE schedules SET is_done = 0, done_date = NULL WHERE id = $1',
          [scheduleId]
        );
      }
    } catch (error) {
      console.error('Failed to toggle schedule:', error);
      throw error;
    }
  }

  async function updateScheduleColor(date: string, color: string): Promise<void> {
    if (!db.value) return;
    
    try {
      // 使用 cell_metadata 表存储颜色，与日程内容完全分离
      await db.value.execute(`
        INSERT INTO cell_metadata (date, cell_color) VALUES ($1, $2)
        ON CONFLICT(date) DO UPDATE SET cell_color = $2
      `, [date, color]);
    } catch (error) {
      console.error('Failed to update schedule color:', error);
      throw error;
    }
  }

  async function getCellColor(date: string): Promise<string> {
    if (!db.value) return '';

    try {
      const result = await db.value.select<{cell_color: string}[]>(
        'SELECT cell_color FROM cell_metadata WHERE date = $1',
        [date]
      );
      return result[0]?.cell_color || '';
    } catch (error) {
      console.error('Failed to get cell color:', error);
      return '';
    }
  }

  async function loadAllSchedules(): Promise<{ schedules: Schedule[], cellColors: Map<string, string> }> {
    if (!db.value) return { schedules: [], cellColors: new Map() };

    try {
      // 获取所有日程
      const schedules = await db.value.select<Schedule[]>(
        'SELECT * FROM schedules ORDER BY create_date, id'
      );

      // 获取所有单元格颜色
      const metadata = await db.value.select<{date: string; cell_color: string}[]>(
        "SELECT date, cell_color FROM cell_metadata WHERE cell_color != ''"
      );

      const cellColors = new Map(metadata.map(m => [m.date, m.cell_color]));

      return { schedules, cellColors };
    } catch (error) {
      console.error('Failed to load all schedules:', error);
      return { schedules: [], cellColors: new Map() };
    }
  }

  async function importSchedules(schedules: Schedule[], merge: boolean = false): Promise<{ inserted: number, updated: number }> {
    if (!db.value) return { inserted: 0, updated: 0 };

    let inserted = 0;
    let updated = 0;

    try {
      if (merge) {
        // 合并模式：智能去重和更新
        for (const schedule of schedules) {
          // 查找是否存在相同 create_date 和 content 的记录
          const existing = await db.value.select<Schedule[]>(
            'SELECT * FROM schedules WHERE create_date = $1 AND content = $2',
            [schedule.create_date, schedule.content]
          );

          if (existing.length > 0) {
            // 存在则更新该记录
            await db.value.execute(
              'UPDATE schedules SET is_done = $1, priority = $2, done_date = $3, description = $4 WHERE id = $5',
              [schedule.is_done ? 1 : 0, schedule.priority, schedule.done_date || null, schedule.description || null, existing[0].id]
            );
            updated++;
          } else {
            // 不存在则插入新记录
            await db.value.execute(
              'INSERT INTO schedules (create_date, content, is_done, priority, done_date, description) VALUES ($1, $2, $3, $4, $5, $6)',
              [schedule.create_date, schedule.content, schedule.is_done ? 1 : 0, schedule.priority, schedule.done_date || null, schedule.description || null]
            );
            inserted++;
          }
        }
      } else {
        // 覆盖模式：直接插入所有记录
        for (const schedule of schedules) {
          await db.value.execute(
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

  async function importCellColors(cellColors: { date: string, color: string }[]): Promise<{ inserted: number, updated: number }> {
    if (!db.value) return { inserted: 0, updated: 0 };

    let inserted = 0;
    let updated = 0;

    try {
      for (const { date, color } of cellColors) {
        // 检查是否已存在该日期的颜色
        const existing = await db.value.select<{ count: number }[]>(
          'SELECT COUNT(*) as count FROM cell_metadata WHERE date = $1',
          [date]
        );

        if (existing[0]?.count > 0) {
          updated++;
        } else {
          inserted++;
        }

        await db.value.execute(
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

  async function clearAllData(): Promise<void> {
    if (!db.value) return;

    try {
      await db.value.execute('DELETE FROM schedules');
      await db.value.execute('DELETE FROM cell_metadata');
    } catch (error) {
      console.error('Failed to clear all data:', error);
      throw error;
    }
  }

  return {
    db,
    initDatabase,
    loadSchedules,
    loadTodoSchedules,
    loadDoneSchedules,
    saveSchedule,
    deleteSchedule,
    deleteSchedulesByDate,
    toggleScheduleStatus,
    updateScheduleColor,
    getCellColor,
    loadAllSchedules,
    importSchedules,
    importCellColors,
    clearAllData,
  };
}
