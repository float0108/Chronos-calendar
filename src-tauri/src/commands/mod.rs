//! Tauri 命令模块
//!
//! 按功能划分：
//! - `app`: 应用级命令（字体、自启动、MCP 服务）
//! - `window`: 窗口管理命令
//! - `database`: 数据库操作命令

pub mod app;
pub mod database;
pub mod window;

// 重导出 McpState 供 lib.rs 使用
pub use app::McpState;
pub use database::DbState;
