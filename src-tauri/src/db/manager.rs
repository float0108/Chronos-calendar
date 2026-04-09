//! 数据库管理器

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// 数据库管理器
#[derive(Debug)]
pub struct DatabaseManager {
    pub(crate) conn: Arc<Mutex<Connection>>,
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
}
