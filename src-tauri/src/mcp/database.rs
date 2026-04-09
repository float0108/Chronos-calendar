//! 数据库操作模块
//!
//! 直接操作 SQLite 数据库，为 MCP 服务提供数据支持

use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use chrono::Datelike;

/// 日程项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleItem {
    pub id: i64,
    pub create_date: Option<String>,
    pub content: String,
    pub is_done: bool,
    pub priority: i32,
    pub done_date: Option<String>,
    pub description: Option<String>,
    pub father_task: Option<i64>,
}

/// 主任务项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainTaskItem {
    pub id: i64,
    pub content: String,
    pub description: Option<String>,
    pub is_done: bool,
    pub priority: i32,
    pub create_date: String,
    pub done_date: Option<String>,
}

/// 单元格元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetadata {
    pub date: String,
    pub cell_color: Option<String>,
}

/// 数据库管理器
#[derive(Debug)]
pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
}

impl DatabaseManager {
    /// 获取数据库路径
    fn get_db_path() -> PathBuf {
        // 开发环境: %APPDATA%/com.chronos.app/chronos.db
        let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(app_data)
            .join("com.chronos.app")
            .join("chronos.db")
    }

    /// 创建新的数据库管理器
    pub fn new() -> Result<Self, String> {
        let db_path = Self::get_db_path();

        // 确保目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database directory: {}", e))?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        // 初始化表结构
        Self::init_tables(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 初始化表结构
    fn init_tables(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schedules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                create_date TEXT,
                content TEXT NOT NULL,
                is_done INTEGER DEFAULT 0,
                priority INTEGER DEFAULT 0,
                done_date TEXT,
                description TEXT,
                father_task INTEGER
            );

            CREATE INDEX IF NOT EXISTS idx_schedules_create_date ON schedules(create_date);

            CREATE TABLE IF NOT EXISTS cell_metadata (
                date TEXT PRIMARY KEY,
                cell_color TEXT DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS main_tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                description TEXT,
                is_done INTEGER DEFAULT 0,
                priority INTEGER DEFAULT 0,
                create_date TEXT NOT NULL,
                done_date TEXT
            );

            CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT DEFAULT '',
                create_date TEXT NOT NULL
            );
            "#,
        ).map_err(|e| format!("Failed to initialize tables: {}", e))?;

        Ok(())
    }

    // ========== 日程操作 ==========

    /// 添加日程
    pub fn add_schedule(&self, item: &ScheduleItem) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        let create_date = item.create_date.as_ref().unwrap_or(&now);

        conn.execute(
            "INSERT INTO schedules (create_date, content, is_done, priority, done_date, description, father_task)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                create_date,
                item.content,
                item.is_done as i32,
                item.priority,
                item.done_date,
                item.description,
                item.father_task
            ],
        ).map_err(|e| format!("Failed to add schedule: {}", e))?;

        Ok(conn.last_insert_rowid())
    }

    /// 批量添加日程
    pub fn add_schedules(&self, items: &[ScheduleItem]) -> Result<Vec<i64>, String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut ids = Vec::new();
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();

        let tx = conn.transaction().map_err(|e| format!("Failed to start transaction: {}", e))?;

        for item in items {
            let create_date = item.create_date.as_ref().unwrap_or(&now);
            tx.execute(
                "INSERT INTO schedules (create_date, content, is_done, priority, done_date, description, father_task)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    create_date,
                    item.content,
                    item.is_done as i32,
                    item.priority,
                    item.done_date,
                    item.description,
                    item.father_task
                ],
            ).map_err(|e| format!("Failed to add schedule: {}", e))?;
            ids.push(tx.last_insert_rowid());
        }

        tx.commit().map_err(|e| format!("Failed to commit transaction: {}", e))?;
        Ok(ids)
    }

    /// 更新日程
    pub fn update_schedule(&self, id: i64, item: &ScheduleItem) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn.execute(
            "UPDATE schedules SET create_date = ?1, content = ?2, is_done = ?3, priority = ?4,
             done_date = ?5, description = ?6, father_task = ?7 WHERE id = ?8",
            params![
                item.create_date,
                item.content,
                item.is_done as i32,
                item.priority,
                item.done_date,
                item.description,
                item.father_task,
                id
            ],
        ).map_err(|e| format!("Failed to update schedule: {}", e))?;

        Ok(affected > 0)
    }

    /// 部分更新日程
    pub fn patch_schedule(&self, id: i64, updates: &SchedulePatch) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut sets = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref v) = updates.create_date {
            sets.push("create_date = ?");
            values.push(Box::new(v.clone()));
        }
        if let Some(ref v) = updates.content {
            sets.push("content = ?");
            values.push(Box::new(v.clone()));
        }
        if let Some(v) = updates.is_done {
            sets.push("is_done = ?");
            values.push(Box::new(v as i32));
        }
        if let Some(v) = updates.priority {
            sets.push("priority = ?");
            values.push(Box::new(v));
        }
        if let Some(ref v) = updates.done_date {
            sets.push("done_date = ?");
            values.push(Box::new(v.clone()));
        }
        if updates.description.is_some() {
            sets.push("description = ?");
            values.push(Box::new(updates.description.clone()));
        }
        if updates.father_task.is_some() {
            sets.push("father_task = ?");
            values.push(Box::new(updates.father_task));
        }

        if sets.is_empty() {
            return Ok(false);
        }

        let sql = format!("UPDATE schedules SET {} WHERE id = ?", sets.join(", "));
        values.push(Box::new(id));

        let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        let affected = conn.execute(&sql, params.as_slice())
            .map_err(|e| format!("Failed to patch schedule: {}", e))?;

        Ok(affected > 0)
    }

    /// 删除日程
    pub fn delete_schedule(&self, id: i64) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn.execute("DELETE FROM schedules WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete schedule: {}", e))?;
        Ok(affected > 0)
    }

    /// 批量删除日程
    pub fn delete_schedules(&self, ids: &[i64]) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!("DELETE FROM schedules WHERE id IN ({})", placeholders.join(","));
        let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let affected = conn.execute(&sql, params.as_slice())
            .map_err(|e| format!("Failed to delete schedules: {}", e))?;
        Ok(affected)
    }

    /// 获取单个日程
    pub fn get_schedule(&self, id: i64) -> Result<Option<ScheduleItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let result = conn.query_row(
            "SELECT id, create_date, content, is_done, priority, done_date, description, father_task
             FROM schedules WHERE id = ?1",
            params![id],
            |row| Ok(ScheduleItem {
                id: row.get(0)?,
                create_date: row.get(1)?,
                content: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                done_date: row.get(5)?,
                description: row.get(6)?,
                father_task: row.get(7)?,
            }),
        ).optional().map_err(|e| format!("Failed to get schedule: {}", e))?;
        Ok(result)
    }

    /// 搜索日程
    pub fn search_schedules(&self, query: &str, limit: Option<usize>) -> Result<Vec<ScheduleItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let limit = limit.unwrap_or(100);
        let search_pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            "SELECT id, create_date, content, is_done, priority, done_date, description, father_task
             FROM schedules
             WHERE content LIKE ?1 OR description LIKE ?1
             ORDER BY create_date DESC, id DESC
             LIMIT ?2"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![search_pattern, limit as i32], |row| {
            Ok(ScheduleItem {
                id: row.get(0)?,
                create_date: row.get(1)?,
                content: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                done_date: row.get(5)?,
                description: row.get(6)?,
                father_task: row.get(7)?,
            })
        }).map_err(|e| format!("Failed to search schedules: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 获取所有日程（带限制）
    pub fn get_all_schedules(&self, limit: Option<usize>) -> Result<Vec<ScheduleItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let limit = limit.unwrap_or(100);

        let mut stmt = conn.prepare(
            "SELECT id, create_date, content, is_done, priority, done_date, description, father_task
             FROM schedules
             ORDER BY create_date DESC, id DESC
             LIMIT ?1"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![limit as i32], |row| {
            Ok(ScheduleItem {
                id: row.get(0)?,
                create_date: row.get(1)?,
                content: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                done_date: row.get(5)?,
                description: row.get(6)?,
                father_task: row.get(7)?,
            })
        }).map_err(|e| format!("Failed to get schedules: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 按日期获取日程
    pub fn get_schedules_by_date(&self, date: &str) -> Result<Vec<ScheduleItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn.prepare(
            "SELECT id, create_date, content, is_done, priority, done_date, description, father_task
             FROM schedules
             WHERE create_date = ?1
             ORDER BY id ASC"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![date], |row| {
            Ok(ScheduleItem {
                id: row.get(0)?,
                create_date: row.get(1)?,
                content: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                done_date: row.get(5)?,
                description: row.get(6)?,
                father_task: row.get(7)?,
            })
        }).map_err(|e| format!("Failed to get schedules: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    // ========== 主任务操作 ==========

    /// 添加主任务
    pub fn add_main_task(&self, item: &MainTaskItem) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        let create_date = if item.create_date.is_empty() { &now } else { &item.create_date };

        conn.execute(
            "INSERT INTO main_tasks (content, description, is_done, priority, create_date, done_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.content,
                item.description,
                item.is_done as i32,
                item.priority,
                create_date,
                item.done_date
            ],
        ).map_err(|e| format!("Failed to add main task: {}", e))?;

        Ok(conn.last_insert_rowid())
    }

    /// 批量添加主任务
    pub fn add_main_tasks(&self, items: &[MainTaskItem]) -> Result<Vec<i64>, String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut ids = Vec::new();
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();

        let tx = conn.transaction().map_err(|e| format!("Failed to start transaction: {}", e))?;

        for item in items {
            let create_date = if item.create_date.is_empty() { &now } else { &item.create_date };
            tx.execute(
                "INSERT INTO main_tasks (content, description, is_done, priority, create_date, done_date)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    item.content,
                    item.description,
                    item.is_done as i32,
                    item.priority,
                    create_date,
                    item.done_date
                ],
            ).map_err(|e| format!("Failed to add main task: {}", e))?;
            ids.push(tx.last_insert_rowid());
        }

        tx.commit().map_err(|e| format!("Failed to commit transaction: {}", e))?;
        Ok(ids)
    }

    /// 更新主任务
    pub fn update_main_task(&self, id: i64, item: &MainTaskItem) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn.execute(
            "UPDATE main_tasks SET content = ?1, description = ?2, is_done = ?3, priority = ?4,
             create_date = ?5, done_date = ?6 WHERE id = ?7",
            params![
                item.content,
                item.description,
                item.is_done as i32,
                item.priority,
                item.create_date,
                item.done_date,
                id
            ],
        ).map_err(|e| format!("Failed to update main task: {}", e))?;

        Ok(affected > 0)
    }

    /// 部分更新主任务
    pub fn patch_main_task(&self, id: i64, updates: &MainTaskPatch) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut sets = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref v) = updates.content {
            sets.push("content = ?");
            values.push(Box::new(v.clone()));
        }
        if updates.description.is_some() {
            sets.push("description = ?");
            values.push(Box::new(updates.description.clone()));
        }
        if let Some(v) = updates.is_done {
            sets.push("is_done = ?");
            values.push(Box::new(v as i32));
        }
        if let Some(v) = updates.priority {
            sets.push("priority = ?");
            values.push(Box::new(v));
        }
        if let Some(ref v) = updates.create_date {
            sets.push("create_date = ?");
            values.push(Box::new(v.clone()));
        }
        if updates.done_date.is_some() {
            sets.push("done_date = ?");
            values.push(Box::new(updates.done_date.clone()));
        }

        if sets.is_empty() {
            return Ok(false);
        }

        let sql = format!("UPDATE main_tasks SET {} WHERE id = ?", sets.join(", "));
        values.push(Box::new(id));

        let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        let affected = conn.execute(&sql, params.as_slice())
            .map_err(|e| format!("Failed to patch main task: {}", e))?;

        Ok(affected > 0)
    }

    /// 删除主任务
    pub fn delete_main_task(&self, id: i64) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn.execute("DELETE FROM main_tasks WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete main task: {}", e))?;
        Ok(affected > 0)
    }

    /// 批量删除主任务
    pub fn delete_main_tasks(&self, ids: &[i64]) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!("DELETE FROM main_tasks WHERE id IN ({})", placeholders.join(","));
        let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let affected = conn.execute(&sql, params.as_slice())
            .map_err(|e| format!("Failed to delete main tasks: {}", e))?;
        Ok(affected)
    }

    /// 获取单个主任务
    pub fn get_main_task(&self, id: i64) -> Result<Option<MainTaskItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let result = conn.query_row(
            "SELECT id, content, description, is_done, priority, create_date, done_date
             FROM main_tasks WHERE id = ?1",
            params![id],
            |row| Ok(MainTaskItem {
                id: row.get(0)?,
                content: row.get(1)?,
                description: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                create_date: row.get(5)?,
                done_date: row.get(6)?,
            }),
        ).optional().map_err(|e| format!("Failed to get main task: {}", e))?;
        Ok(result)
    }

    /// 搜索主任务
    pub fn search_main_tasks(&self, query: &str, limit: Option<usize>) -> Result<Vec<MainTaskItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let limit = limit.unwrap_or(100);
        let search_pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            "SELECT id, content, description, is_done, priority, create_date, done_date
             FROM main_tasks
             WHERE content LIKE ?1 OR description LIKE ?1
             ORDER BY is_done ASC, create_date DESC, id DESC
             LIMIT ?2"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![search_pattern, limit as i32], |row| {
            Ok(MainTaskItem {
                id: row.get(0)?,
                content: row.get(1)?,
                description: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                create_date: row.get(5)?,
                done_date: row.get(6)?,
            })
        }).map_err(|e| format!("Failed to search main tasks: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 获取所有主任务（带限制）
    pub fn get_all_main_tasks(&self, limit: Option<usize>) -> Result<Vec<MainTaskItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let limit = limit.unwrap_or(100);

        let mut stmt = conn.prepare(
            "SELECT id, content, description, is_done, priority, create_date, done_date
             FROM main_tasks
             ORDER BY is_done ASC, create_date DESC, id DESC
             LIMIT ?1"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![limit as i32], |row| {
            Ok(MainTaskItem {
                id: row.get(0)?,
                content: row.get(1)?,
                description: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                create_date: row.get(5)?,
                done_date: row.get(6)?,
            })
        }).map_err(|e| format!("Failed to get main tasks: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 获取今日日程统计
    pub fn get_today_stats(&self) -> Result<TodayStats, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let total: i32 = conn.query_row(
            "SELECT COUNT(*) FROM schedules WHERE create_date = ?1",
            params![today],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count schedules: {}", e))?;

        let completed: i32 = conn.query_row(
            "SELECT COUNT(*) FROM schedules WHERE create_date = ?1 AND is_done = 1",
            params![today],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count completed schedules: {}", e))?;

        Ok(TodayStats { total, completed })
    }

    /// 获取本周日程统计
    pub fn get_week_stats(&self) -> Result<WeekStats, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let today = chrono::Local::now();
        let week_start = (today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64))
            .format("%Y-%m-%d").to_string();
        let week_end = (today + chrono::Duration::days(6 - today.weekday().num_days_from_monday() as i64))
            .format("%Y-%m-%d").to_string();

        let total: i32 = conn.query_row(
            "SELECT COUNT(*) FROM schedules WHERE create_date >= ?1 AND create_date <= ?2",
            params![week_start, week_end],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count schedules: {}", e))?;

        let completed: i32 = conn.query_row(
            "SELECT COUNT(*) FROM schedules WHERE create_date >= ?1 AND create_date <= ?2 AND is_done = 1",
            params![week_start, week_end],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count completed schedules: {}", e))?;

        Ok(WeekStats { total, completed, week_start, week_end })
    }

    /// 获取待办日程（未完成的日程）
    pub fn get_pending_schedules(&self, limit: Option<usize>) -> Result<Vec<ScheduleItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let limit = limit.unwrap_or(100);

        let mut stmt = conn.prepare(
            "SELECT id, create_date, content, is_done, priority, done_date, description, father_task
             FROM schedules
             WHERE is_done = 0
             ORDER BY priority DESC, create_date ASC
             LIMIT ?1"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![limit as i32], |row| {
            Ok(ScheduleItem {
                id: row.get(0)?,
                create_date: row.get(1)?,
                content: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                done_date: row.get(5)?,
                description: row.get(6)?,
                father_task: row.get(7)?,
            })
        }).map_err(|e| format!("Failed to get pending schedules: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 获取进行中的主任务（未完成的）
    pub fn get_active_main_tasks(&self, limit: Option<usize>) -> Result<Vec<MainTaskItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let limit = limit.unwrap_or(100);

        let mut stmt = conn.prepare(
            "SELECT id, content, description, is_done, priority, create_date, done_date
             FROM main_tasks
             WHERE is_done = 0
             ORDER BY priority DESC, create_date ASC
             LIMIT ?1"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![limit as i32], |row| {
            Ok(MainTaskItem {
                id: row.get(0)?,
                content: row.get(1)?,
                description: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                create_date: row.get(5)?,
                done_date: row.get(6)?,
            })
        }).map_err(|e| format!("Failed to get active main tasks: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }
}

/// 今日统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodayStats {
    pub total: i32,
    pub completed: i32,
}

/// 本周统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekStats {
    pub total: i32,
    pub completed: i32,
    pub week_start: String,
    pub week_end: String,
}

/// 日程部分更新结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchedulePatch {
    pub create_date: Option<String>,
    pub content: Option<String>,
    pub is_done: Option<bool>,
    pub priority: Option<i32>,
    pub done_date: Option<String>,
    pub description: Option<String>,
    pub father_task: Option<i64>,
}

/// 主任务部分更新结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MainTaskPatch {
    pub content: Option<String>,
    pub description: Option<String>,
    pub is_done: Option<bool>,
    pub priority: Option<i32>,
    pub create_date: Option<String>,
    pub done_date: Option<String>,
}
