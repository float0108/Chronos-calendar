import { ref } from 'vue';
import Database from '@tauri-apps/plugin-sql';

const db = ref<Database | null>(null);

/**
 * 初始化数据库连接
 */
export function getDatabase(): Database | null {
  return db.value;
}

/**
 * 初始化数据库和迁移
 */
export async function initDatabase(): Promise<void> {
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
    } catch {
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
