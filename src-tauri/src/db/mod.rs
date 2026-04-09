//! 数据库模块
//!
//! 提供 SQLite 数据库操作，为 Tauri commands 和 MCP 服务提供统一的数据访问层

mod types;
mod manager;
mod schedule;
mod main_task;
mod note;
mod cell;
mod stats;
mod backup;

// 导出类型
pub use types::*;

// 导出管理器
pub use manager::DatabaseManager;
