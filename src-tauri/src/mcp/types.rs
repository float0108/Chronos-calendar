//! MCP 请求/响应类型定义
//!
//! 定义所有工具的请求结构体

use rmcp::schemars;
use serde::Deserialize;

// ========== 日程相关请求 ==========

/// 添加日程请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddScheduleRequest {
    #[schemars(description = "日程内容（必填）")]
    pub content: String,
    #[schemars(description = "计划日期 (YYYY-MM-DD 格式)，默认为今天")]
    pub create_date: Option<String>,
    #[schemars(description = "是否已完成，默认 false")]
    pub is_done: Option<bool>,
    #[schemars(description = "优先级 (0-9)，默认 0")]
    pub priority: Option<i32>,
    #[schemars(description = "详细描述/备注")]
    pub description: Option<String>,
    #[schemars(description = "完成日期 (YYYY-MM-DD 格式)")]
    pub done_date: Option<String>,
    #[schemars(description = "关联的主任务 ID")]
    pub father_task: Option<i64>,
}

/// 批量添加日程请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddSchedulesRequest {
    #[schemars(description = "日程项列表")]
    pub items: Vec<AddScheduleRequest>,
}

/// 更新日程请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateScheduleRequest {
    #[schemars(description = "日程 ID（必填）")]
    pub id: i64,
    #[schemars(description = "日程内容（必填）")]
    pub content: String,
    #[schemars(description = "计划日期")]
    pub create_date: Option<String>,
    #[schemars(description = "是否已完成")]
    pub is_done: Option<bool>,
    #[schemars(description = "优先级")]
    pub priority: Option<i32>,
    #[schemars(description = "详细描述")]
    pub description: Option<String>,
    #[schemars(description = "完成日期")]
    pub done_date: Option<String>,
    #[schemars(description = "关联的主任务 ID")]
    pub father_task: Option<i64>,
}

/// 部分更新日程请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PatchScheduleRequest {
    #[schemars(description = "日程 ID（必填）")]
    pub id: i64,
    #[schemars(description = "计划日期")]
    pub create_date: Option<String>,
    #[schemars(description = "日程内容")]
    pub content: Option<String>,
    #[schemars(description = "是否已完成")]
    pub is_done: Option<bool>,
    #[schemars(description = "优先级")]
    pub priority: Option<i32>,
    #[schemars(description = "详细描述")]
    pub description: Option<String>,
    #[schemars(description = "完成日期")]
    pub done_date: Option<String>,
    #[schemars(description = "关联的主任务 ID")]
    pub father_task: Option<i64>,
}

/// 删除日程请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteScheduleRequest {
    #[schemars(description = "日程 ID（必填）")]
    pub id: i64,
}

/// 批量删除日程请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteSchedulesRequest {
    #[schemars(description = "日程 ID 列表（必填）")]
    pub ids: Vec<i64>,
}

/// 获取日程请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetScheduleRequest {
    #[schemars(description = "日程 ID（必填）")]
    pub id: i64,
}

/// 搜索日程请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchSchedulesRequest {
    #[schemars(description = "搜索关键词（必填）")]
    pub query: String,
    #[schemars(description = "最大返回数量，默认 100")]
    pub limit: Option<usize>,
}

/// 列出日程请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListSchedulesRequest {
    #[schemars(description = "最大返回数量，默认 100")]
    pub limit: Option<usize>,
}

/// 按日期获取日程请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetSchedulesByDateRequest {
    #[schemars(description = "日期 (YYYY-MM-DD 格式，必填)")]
    pub date: String,
}

// ========== 主任务相关请求 ==========

/// 添加主任务请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddMainTaskRequest {
    #[schemars(description = "任务内容（必填）")]
    pub content: String,
    #[schemars(description = "详细描述")]
    pub description: Option<String>,
    #[schemars(description = "优先级 (0-9)，默认 0")]
    pub priority: Option<i32>,
    #[schemars(description = "创建日期，默认为今天")]
    pub create_date: Option<String>,
    #[schemars(description = "是否已完成，默认 false")]
    pub is_done: Option<bool>,
    #[schemars(description = "完成日期")]
    pub done_date: Option<String>,
}

/// 批量添加主任务请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddMainTasksRequest {
    #[schemars(description = "任务项列表")]
    pub items: Vec<AddMainTaskRequest>,
}

/// 更新主任务请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateMainTaskRequest {
    #[schemars(description = "任务 ID（必填）")]
    pub id: i64,
    #[schemars(description = "任务内容（必填）")]
    pub content: String,
    #[schemars(description = "详细描述")]
    pub description: Option<String>,
    #[schemars(description = "优先级")]
    pub priority: Option<i32>,
    #[schemars(description = "创建日期")]
    pub create_date: Option<String>,
    #[schemars(description = "是否已完成")]
    pub is_done: Option<bool>,
    #[schemars(description = "完成日期")]
    pub done_date: Option<String>,
}

/// 部分更新主任务请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PatchMainTaskRequest {
    #[schemars(description = "任务 ID（必填）")]
    pub id: i64,
    #[schemars(description = "任务内容")]
    pub content: Option<String>,
    #[schemars(description = "详细描述")]
    pub description: Option<String>,
    #[schemars(description = "优先级")]
    pub priority: Option<i32>,
    #[schemars(description = "创建日期")]
    pub create_date: Option<String>,
    #[schemars(description = "是否已完成")]
    pub is_done: Option<bool>,
    #[schemars(description = "完成日期")]
    pub done_date: Option<String>,
}

/// 删除主任务请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteMainTaskRequest {
    #[schemars(description = "任务 ID（必填）")]
    pub id: i64,
}

/// 批量删除主任务请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteMainTasksRequest {
    #[schemars(description = "任务 ID 列表（必填）")]
    pub ids: Vec<i64>,
}

/// 获取主任务请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetMainTaskRequest {
    #[schemars(description = "任务 ID（必填）")]
    pub id: i64,
}

/// 搜索主任务请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchMainTasksRequest {
    #[schemars(description = "搜索关键词（必填）")]
    pub query: String,
    #[schemars(description = "最大返回数量，默认 100")]
    pub limit: Option<usize>,
}

/// 列出主任务请求
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListMainTasksRequest {
    #[schemars(description = "最大返回数量，默认 100")]
    pub limit: Option<usize>,
}
