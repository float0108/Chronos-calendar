import { ref } from 'vue';
import Database from '@tauri-apps/plugin-sql';

const db = ref<Database | null>(null);

/**
 * 获取数据库实例
 */
export function getDatabase(): Database | null {
  return db.value;
}

/**
 * 初始化数据库
 */
export async function initDatabase(): Promise<void> {
  try {
    db.value = await Database.load('sqlite:chronos.db');

    // 日程表
    await db.value!.execute(`
      CREATE TABLE IF NOT EXISTS schedules (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        create_date TEXT,
        content TEXT NOT NULL,
        is_done INTEGER DEFAULT 0,
        priority INTEGER DEFAULT 0,
        done_date TEXT,
        description TEXT,
        father_task INTEGER,
        FOREIGN KEY (father_task) REFERENCES main_tasks(id)
      )
    `);

    await db.value!.execute(`
      CREATE INDEX IF NOT EXISTS idx_schedules_create_date ON schedules(create_date)
    `);

    // 单元格元数据表 - 存储颜色等元数据
    await db.value!.execute(`
      CREATE TABLE IF NOT EXISTS cell_metadata (
        date TEXT PRIMARY KEY,
        cell_color TEXT DEFAULT ''
      )
    `);

    // 主任务表 - 独立的待办任务列表
    await db.value!.execute(`
      CREATE TABLE IF NOT EXISTS main_tasks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        content TEXT NOT NULL,
        description TEXT,
        is_done INTEGER DEFAULT 0,
        priority INTEGER DEFAULT 0,
        create_date TEXT NOT NULL,
        done_date TEXT
      )
    `);

    // 备忘录表
    await db.value!.execute(`
      CREATE TABLE IF NOT EXISTS notes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        content TEXT DEFAULT '',
        create_date TEXT NOT NULL
      )
    `);
  } catch (error) {
    console.error('Database initialization failed:', error);
    throw error;
  }
}
