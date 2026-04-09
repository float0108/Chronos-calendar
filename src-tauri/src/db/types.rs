//! 数据库类型定义

use serde::{Deserialize, Serialize};

// ========== 日程 ==========

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

// ========== 主任务 ==========

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

// ========== 备忘录 ==========

/// 备忘录项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteItem {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub create_date: String,
}

/// 备忘录部分更新结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotePatch {
    pub title: Option<String>,
    pub content: Option<String>,
    pub create_date: Option<String>,
}

// ========== 单元格元数据 ==========

/// 单元格元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetadata {
    pub date: String,
    pub cell_color: Option<String>,
}

// ========== 统计 ==========

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

// ========== 备份 ==========

/// 备份数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub schedules: Vec<ScheduleItem>,
    pub main_tasks: Vec<MainTaskItem>,
    pub notes: Vec<NoteItem>,
    pub cell_metadata: Vec<CellMetadata>,
}

/// 导入统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportStats {
    pub schedules: TableStats,
    pub main_tasks: TableStats,
    pub notes: TableStats,
    pub cell_metadata: TableStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableStats {
    pub inserted: usize,
    pub updated: usize,
}
