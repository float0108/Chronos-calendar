import { ref } from 'vue';
import Database from '@tauri-apps/plugin-sql';
import type { Schedule } from '../types';

const db = ref<Database | null>(null);

export function useDatabase() {
  async function initDatabase(): Promise<void> {
    try {
      db.value = await Database.load('sqlite:chronos.db');
      
      // 日程表 - 只存储实际日程内容
      await db.value.execute(`
        CREATE TABLE IF NOT EXISTS schedules (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          date TEXT NOT NULL,
          content TEXT NOT NULL,
          is_done INTEGER DEFAULT 0,
          priority INTEGER DEFAULT 0,
          created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
      `);
      
      await db.value.execute(`
        CREATE INDEX IF NOT EXISTS idx_schedules_date ON schedules(date)
      `);
      
      // 单元格元数据表 - 存储颜色等元数据（与内容分离）
      await db.value.execute(`
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
          SELECT date, cell_color FROM schedules 
          WHERE cell_color != '' AND cell_color IS NOT NULL
          GROUP BY date
        `);
        // 删除旧列（SQLite 不支持直接删除列，这里保留但不使用）
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

  async function loadSchedules(monthStr: string): Promise<Schedule[]> {
    if (!db.value) return [];
    
    try {
      // 获取日程内容
      const schedules = await db.value.select<Schedule[]>(`
        SELECT * FROM schedules 
        WHERE date LIKE $1 || '%'
        ORDER BY created_at ASC
      `, [monthStr]);
      
      // 获取颜色元数据
      const metadata = await db.value.select<{date: string; cell_color: string}[]>(`
        SELECT * FROM cell_metadata 
        WHERE date LIKE $1 || '%'
      `, [monthStr]);
      
      // 创建颜色映射
      const colorMap = new Map(metadata.map(m => [m.date, m.cell_color]));
      
      // 按日期分组处理日程
      const grouped = new Map<string, Schedule[]>();
      schedules.forEach(s => {
        if (!grouped.has(s.date)) grouped.set(s.date, []);
        grouped.get(s.date)!.push(s);
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
            date,
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

  async function saveSchedule(date: string, content: string, isDone: boolean = false): Promise<void> {
    if (!db.value) return;
    
    try {
      await db.value.execute(
        'INSERT INTO schedules (date, content, is_done, priority) VALUES ($1, $2, $3, $4)',
        [date, content.trim(), isDone ? 1 : 0, 0]
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
      await db.value.execute('DELETE FROM schedules WHERE date = $1', [date]);
    } catch (error) {
      console.error('Failed to delete schedules:', error);
      throw error;
    }
  }

  async function toggleScheduleStatus(scheduleId: number, isDone: boolean): Promise<void> {
    if (!db.value) return;
    
    try {
      await db.value.execute(
        'UPDATE schedules SET is_done = $1 WHERE id = $2',
        [isDone ? 0 : 1, scheduleId]
      );
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

  return {
    db,
    initDatabase,
    loadSchedules,
    saveSchedule,
    deleteSchedule,
    deleteSchedulesByDate,
    toggleScheduleStatus,
    updateScheduleColor,
    getCellColor,
  };
}
