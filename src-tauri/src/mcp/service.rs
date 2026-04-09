//! MCP 服务业务逻辑实现
//!
//! 实现 ChronosMcpService 及其所有工具方法

use std::sync::Arc;

use rmcp::{schemars, tool, ServerHandler};
use serde_json::json;
use tauri::{AppHandle, Emitter};

use crate::db::{DatabaseManager, MainTaskItem, MainTaskPatch, ScheduleItem, SchedulePatch};
use super::types::*;

/// Chronos MCP 服务
#[derive(Debug, Clone)]
pub struct ChronosMcpService {
    pub db: Arc<DatabaseManager>,
    pub peer: Option<rmcp::Peer<rmcp::RoleServer>>,
    pub app_handle: Option<Arc<AppHandle>>,
}

impl ChronosMcpService {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            db,
            peer: None,
            app_handle: None,
        }
    }

    pub fn with_app_handle(mut self, app_handle: Arc<AppHandle>) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    /// 通知所有窗口数据已变更
    pub fn notify_data_changed(&self) {
        if let Some(ref app_handle) = self.app_handle {
            // 向所有窗口发送 schedule-changed 事件
            let _ = app_handle.emit("schedule-changed", ());
            println!("[MCP] Notified all windows: schedule-changed");
        }
    }

    /// 获取今日概览
    pub fn get_today_summary(&self) -> String {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        match self.db.get_today_stats() {
            Ok(stats) => {
                let pending = stats.total - stats.completed;
                format!(
                    "📅 今日概览 ({})\n总日程: {} 个\n已完成: {} 个\n待办: {} 个",
                    today, stats.total, stats.completed, pending
                )
            }
            Err(e) => format!("获取今日概览失败: {}", e),
        }
    }

    /// 获取本周概览
    pub fn get_week_summary(&self) -> String {
        match self.db.get_week_stats() {
            Ok(stats) => {
                let pending = stats.total - stats.completed;
                format!(
                    "📅 本周概览 ({} ~ {})\n总日程: {} 个\n已完成: {} 个\n待办: {} 个",
                    stats.week_start, stats.week_end, stats.total, stats.completed, pending
                )
            }
            Err(e) => format!("获取本周概览失败: {}", e),
        }
    }

    /// 获取待办日程列表
    pub fn get_pending_schedules(&self) -> String {
        match self.db.get_pending_schedules(Some(20)) {
            Ok(items) => {
                if items.is_empty() {
                    return "✅ 没有待办日程".to_string();
                }
                let mut result = format!("📋 待办日程 (共 {} 项)\n\n", items.len());
                for (i, item) in items.iter().enumerate() {
                    let date = item.create_date.as_deref().unwrap_or("未设定日期");
                    result.push_str(&format!(
                        "{}. [{}] {} (优先级: {})\n   日期: {}\n",
                        i + 1,
                        if item.is_done { "✓" } else { "○" },
                        item.content,
                        item.priority,
                        date
                    ));
                }
                result
            }
            Err(e) => format!("获取待办日程失败: {}", e),
        }
    }

    /// 获取进行中的主任务
    pub fn get_active_main_tasks(&self) -> String {
        match self.db.get_active_main_tasks(Some(20)) {
            Ok(items) => {
                if items.is_empty() {
                    return "✅ 没有进行中的主任务".to_string();
                }
                let mut result = format!("🎯 进行中的主任务 (共 {} 项)\n\n", items.len());
                for (i, item) in items.iter().enumerate() {
                    result.push_str(&format!(
                        "{}. {} (优先级: {})\n   创建于: {}\n",
                        i + 1,
                        item.content,
                        item.priority,
                        item.create_date
                    ));
                    if let Some(ref desc) = item.description {
                        if !desc.is_empty() {
                            result.push_str(&format!("   描述: {}\n", desc));
                        }
                    }
                }
                result
            }
            Err(e) => format!("获取主任务失败: {}", e),
        }
    }
}

// ========== 工具实现 ==========

#[tool(tool_box)]
impl ChronosMcpService {
    // ========== 日程工具 ==========

    #[tool(description = "添加单个日程项。返回新创建的日程 ID。")]
    pub async fn add_schedule(&self, #[tool(aggr)] req: AddScheduleRequest) -> String {
        println!("[MCP] Tool called: add_schedule, content: {}", req.content);
        let item = ScheduleItem {
            id: 0,
            content: req.content,
            create_date: req.create_date,
            is_done: req.is_done.unwrap_or(false),
            priority: req.priority.unwrap_or(0),
            description: req.description,
            done_date: req.done_date,
            father_task: req.father_task,
        };
        match self.db.add_schedule(&item) {
            Ok(id) => {
                println!("[MCP] Schedule added successfully, id: {}", id);
                self.notify_data_changed();
                json!({ "success": true, "id": id, "message": "日程添加成功" }).to_string()
            }
            Err(e) => {
                println!("[MCP] Failed to add schedule: {}", e);
                json!({ "success": false, "error": e }).to_string()
            }
        }
    }

    #[tool(description = "批量添加日程项。返回新创建的日程 ID 列表。")]
    pub async fn add_schedules(&self, #[tool(aggr)] req: AddSchedulesRequest) -> String {
        let items: Vec<ScheduleItem> = req.items.into_iter().map(|r| ScheduleItem {
            id: 0,
            content: r.content,
            create_date: r.create_date,
            is_done: r.is_done.unwrap_or(false),
            priority: r.priority.unwrap_or(0),
            description: r.description,
            done_date: r.done_date,
            father_task: r.father_task,
        }).collect();
        match self.db.add_schedules(&items) {
            Ok(ids) => {
                self.notify_data_changed();
                json!({ "success": true, "ids": ids, "count": ids.len(), "message": "批量添加日程成功" }).to_string()
            }
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "完整更新日程项的所有字段。返回是否成功。")]
    pub async fn update_schedule(&self, #[tool(aggr)] req: UpdateScheduleRequest) -> String {
        let item = ScheduleItem {
            id: 0,
            content: req.content,
            create_date: req.create_date,
            is_done: req.is_done.unwrap_or(false),
            priority: req.priority.unwrap_or(0),
            description: req.description,
            done_date: req.done_date,
            father_task: req.father_task,
        };
        match self.db.update_schedule(req.id, &item) {
            Ok(success) => {
                if success { self.notify_data_changed(); }
                json!({
                    "success": success,
                    "message": if success { "日程更新成功" } else { "日程不存在" }
                }).to_string()
            }
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "部分更新日程项，只修改指定字段。返回是否成功。")]
    pub async fn patch_schedule(&self, #[tool(aggr)] req: PatchScheduleRequest) -> String {
        let patch = SchedulePatch {
            create_date: req.create_date,
            content: req.content,
            is_done: req.is_done,
            priority: req.priority,
            done_date: req.done_date,
            description: req.description,
            father_task: req.father_task,
        };
        match self.db.patch_schedule(req.id, &patch) {
            Ok(success) => {
                if success { self.notify_data_changed(); }
                json!({
                    "success": success,
                    "message": if success { "日程更新成功" } else { "日程不存在或无更新" }
                }).to_string()
            }
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "删除单个日程项。返回是否成功。")]
    pub async fn delete_schedule(&self, #[tool(aggr)] req: DeleteScheduleRequest) -> String {
        match self.db.delete_schedule(req.id) {
            Ok(success) => {
                if success { self.notify_data_changed(); }
                json!({
                    "success": success,
                    "message": if success { "日程删除成功" } else { "日程不存在" }
                }).to_string()
            }
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "批量删除日程项。返回删除的数量。")]
    pub async fn delete_schedules(&self, #[tool(aggr)] req: DeleteSchedulesRequest) -> String {
        match self.db.delete_schedules(&req.ids) {
            Ok(count) => {
                if count > 0 { self.notify_data_changed(); }
                json!({ "success": true, "deleted_count": count, "message": format!("已删除 {} 个日程", count) }).to_string()
            }
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "根据 ID 获取单个日程项详情。")]
    pub async fn get_schedule(&self, #[tool(aggr)] req: GetScheduleRequest) -> String {
        match self.db.get_schedule(req.id) {
            Ok(Some(item)) => json!({ "success": true, "data": item }).to_string(),
            Ok(None) => json!({ "success": false, "message": "日程不存在" }).to_string(),
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "搜索日程项（按内容或描述）。支持限制返回数量。")]
    pub async fn search_schedules(&self, #[tool(aggr)] req: SearchSchedulesRequest) -> String {
        match self.db.search_schedules(&req.query, req.limit) {
            Ok(items) => json!({ "success": true, "data": items, "count": items.len() }).to_string(),
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "获取所有日程项列表。支持限制返回数量。")]
    pub async fn list_schedules(&self, #[tool(aggr)] req: ListSchedulesRequest) -> String {
        match self.db.get_all_schedules(req.limit) {
            Ok(items) => json!({ "success": true, "data": items, "count": items.len() }).to_string(),
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "获取指定日期的所有日程项。")]
    pub async fn get_schedules_by_date(&self, #[tool(aggr)] req: GetSchedulesByDateRequest) -> String {
        match self.db.get_schedules_by_date(&req.date) {
            Ok(items) => json!({ "success": true, "data": items, "count": items.len() }).to_string(),
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "根据主任务 ID 获取所有关联的子日程项。")]
    pub async fn get_schedules_by_father_task(&self, #[tool(aggr)] req: GetSchedulesByFatherTaskRequest) -> String {
        match self.db.get_schedules_by_father_task(req.father_task_id) {
            Ok(items) => json!({ "success": true, "data": items, "count": items.len() }).to_string(),
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    // ========== 主任务工具 ==========

    #[tool(description = "添加单个主任务。返回新创建的任务 ID。")]
    pub async fn add_main_task(&self, #[tool(aggr)] req: AddMainTaskRequest) -> String {
        let item = MainTaskItem {
            id: 0,
            content: req.content,
            description: req.description,
            is_done: req.is_done.unwrap_or(false),
            priority: req.priority.unwrap_or(0),
            create_date: req.create_date.unwrap_or_default(),
            done_date: req.done_date,
        };
        match self.db.add_main_task(&item) {
            Ok(id) => {
                self.notify_data_changed();
                json!({ "success": true, "id": id, "message": "主任务添加成功" }).to_string()
            }
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "批量添加主任务。返回新创建的任务 ID 列表。")]
    pub async fn add_main_tasks(&self, #[tool(aggr)] req: AddMainTasksRequest) -> String {
        let items: Vec<MainTaskItem> = req.items.into_iter().map(|r| MainTaskItem {
            id: 0,
            content: r.content,
            description: r.description,
            is_done: r.is_done.unwrap_or(false),
            priority: r.priority.unwrap_or(0),
            create_date: r.create_date.unwrap_or_default(),
            done_date: r.done_date,
        }).collect();
        match self.db.add_main_tasks(&items) {
            Ok(ids) => {
                self.notify_data_changed();
                json!({ "success": true, "ids": ids, "count": ids.len(), "message": "批量添加主任务成功" }).to_string()
            }
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "完整更新主任务的所有字段。返回是否成功。")]
    pub async fn update_main_task(&self, #[tool(aggr)] req: UpdateMainTaskRequest) -> String {
        let item = MainTaskItem {
            id: 0,
            content: req.content,
            description: req.description,
            is_done: req.is_done.unwrap_or(false),
            priority: req.priority.unwrap_or(0),
            create_date: req.create_date.unwrap_or_default(),
            done_date: req.done_date,
        };
        match self.db.update_main_task(req.id, &item) {
            Ok(success) => {
                if success { self.notify_data_changed(); }
                json!({
                    "success": success,
                    "message": if success { "主任务更新成功" } else { "主任务不存在" }
                }).to_string()
            }
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "部分更新主任务，只修改指定字段。返回是否成功。")]
    pub async fn patch_main_task(&self, #[tool(aggr)] req: PatchMainTaskRequest) -> String {
        let patch = MainTaskPatch {
            content: req.content,
            description: req.description,
            is_done: req.is_done,
            priority: req.priority,
            create_date: req.create_date,
            done_date: req.done_date,
        };
        match self.db.patch_main_task(req.id, &patch) {
            Ok(success) => {
                if success { self.notify_data_changed(); }
                json!({
                    "success": success,
                    "message": if success { "主任务更新成功" } else { "主任务不存在或无更新" }
                }).to_string()
            }
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "删除单个主任务。返回是否成功。")]
    pub async fn delete_main_task(&self, #[tool(aggr)] req: DeleteMainTaskRequest) -> String {
        match self.db.delete_main_task(req.id) {
            Ok(success) => {
                if success { self.notify_data_changed(); }
                json!({
                    "success": success,
                    "message": if success { "主任务删除成功" } else { "主任务不存在" }
                }).to_string()
            }
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "批量删除主任务。返回删除的数量。")]
    pub async fn delete_main_tasks(&self, #[tool(aggr)] req: DeleteMainTasksRequest) -> String {
        match self.db.delete_main_tasks(&req.ids) {
            Ok(count) => {
                if count > 0 { self.notify_data_changed(); }
                json!({ "success": true, "deleted_count": count, "message": format!("已删除 {} 个主任务", count) }).to_string()
            }
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "根据 ID 获取单个主任务详情。")]
    pub async fn get_main_task(&self, #[tool(aggr)] req: GetMainTaskRequest) -> String {
        match self.db.get_main_task(req.id) {
            Ok(Some(item)) => json!({ "success": true, "data": item }).to_string(),
            Ok(None) => json!({ "success": false, "message": "主任务不存在" }).to_string(),
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "搜索主任务（按内容或描述）。支持限制返回数量。")]
    pub async fn search_main_tasks(&self, #[tool(aggr)] req: SearchMainTasksRequest) -> String {
        match self.db.search_main_tasks(&req.query, req.limit) {
            Ok(items) => json!({ "success": true, "data": items, "count": items.len() }).to_string(),
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }

    #[tool(description = "获取所有主任务列表。支持限制返回数量。")]
    pub async fn list_main_tasks(&self, #[tool(aggr)] req: ListMainTasksRequest) -> String {
        match self.db.get_all_main_tasks(req.limit) {
            Ok(items) => json!({ "success": true, "data": items, "count": items.len() }).to_string(),
            Err(e) => json!({ "success": false, "error": e }).to_string(),
        }
    }
}

// ========== ServerHandler 实现 ==========

#[tool(tool_box)]
impl ServerHandler for ChronosMcpService {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        use rmcp::model::ServerCapabilities;
        rmcp::model::ServerInfo {
            instructions: Some("Chronos 日历应用 MCP 服务。提供日程和主任务的 CRUD 操作。".into()),
            capabilities: ServerCapabilities::builder()
                .enable_resources()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }

    fn get_peer(&self) -> Option<rmcp::Peer<rmcp::RoleServer>> {
        self.peer.clone()
    }

    fn set_peer(&mut self, peer: rmcp::Peer<rmcp::RoleServer>) {
        self.peer = Some(peer);
    }

    /// 列出所有可用资源
    async fn list_resources(
        &self,
        _pagination: rmcp::model::PaginatedRequestParam,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListResourcesResult, rmcp::model::ErrorData> {
        use rmcp::model::{Annotated, ListResourcesResult, RawResource};

        println!("[MCP] Tool called: list_resources");

        Ok(ListResourcesResult {
            resources: vec![
                Annotated::new(RawResource::new("chronos://summary/today", "今日概览"), None),
                Annotated::new(RawResource::new("chronos://summary/week", "本周概览"), None),
                Annotated::new(RawResource::new("chronos://schedules/pending", "待办日程"), None),
                Annotated::new(RawResource::new("chronos://main_tasks/active", "进行中的主任务"), None),
            ],
            next_cursor: None,
        })
    }

    /// 读取指定资源
    async fn read_resource(
        &self,
        request: rmcp::model::ReadResourceRequestParam,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ReadResourceResult, rmcp::model::ErrorData> {
        use rmcp::model::{ErrorData, ReadResourceResult, ResourceContents};

        println!("[MCP] Tool called: read_resource, uri: {}", request.uri);

        let content = match request.uri.as_str() {
            "chronos://summary/today" => self.get_today_summary(),
            "chronos://summary/week" => self.get_week_summary(),
            "chronos://schedules/pending" => self.get_pending_schedules(),
            "chronos://main_tasks/active" => self.get_active_main_tasks(),
            _ => return Err(ErrorData::resource_not_found(
                format!("Resource not found: {}", request.uri),
                None,
            )),
        };

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::text(content, request.uri)],
        })
    }
}
