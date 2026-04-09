//! MCP (Model Context Protocol) 服务模块
//!
//! 提供标准化的 MCP 接口，允许 AI 助手访问和管理日程/任务数据

mod database;
mod server;
mod service;
mod tools;
mod types;

pub use server::{start_mcp_server, McpServerHandle};
pub use service::ChronosMcpService;
pub use types::*;
