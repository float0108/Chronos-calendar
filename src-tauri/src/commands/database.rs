//! 数据库 Tauri 命令
//!
//! 提供前端调用的数据库操作接口

use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tauri::{State, AppHandle, Emitter};

use crate::db::*;

/// 数据变更事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChange {
    pub entity: String,  // "schedule" | "main_task" | "note" | "cell_color"
    pub action: String, // "created" | "updated" | "deleted"
    pub id: i64,
    pub data: Option<serde_json::Value>, // full data for created/updated
}

/// 数据库状态
pub struct DbState {
    pub manager: Arc<DatabaseManager>,
    pub app_handle: Option<AppHandle>,
}

impl DbState {
    /// 发送数据变更事件到所有窗口
    fn notify_change(&self, change: DataChange) {
        if let Some(ref handle) = self.app_handle {
            let _ = handle.emit("schedule-changed", change);
        }
    }
}

// ========== Schedule Commands ==========

#[tauri::command]
pub async fn db_load_schedules(
    state: State<'_, DbState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ScheduleItem>, String> {
    state.manager.get_schedules_by_range(&start_date, &end_date)
}

#[tauri::command]
pub async fn db_load_todo_schedules(
    state: State<'_, DbState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ScheduleItem>, String> {
    state.manager.get_todo_schedules_by_range(&start_date, &end_date)
}

#[tauri::command]
pub async fn db_load_done_schedules(
    state: State<'_, DbState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ScheduleItem>, String> {
    state.manager.get_done_schedules_by_range(&start_date, &end_date)
}

#[tauri::command]
pub async fn db_save_schedule(
    state: State<'_, DbState>,
    create_date: String,
    content: String,
    is_done: Option<bool>,
    done_date: Option<String>,
    description: Option<String>,
    father_task: Option<i64>,
) -> Result<i64, String> {
    let item = ScheduleItem {
        id: 0,
        create_date: Some(create_date),
        content,
        is_done: is_done.unwrap_or(false),
        priority: 0,
        done_date,
        description,
        father_task,
    };
    let id = state.manager.add_schedule(&item)?;
    let change = DataChange {
        entity: "schedule".into(),
        action: "created".into(),
        id,
        data: Some(serde_json::to_value(&item).unwrap()),
    };
    state.notify_change(change);
    Ok(id)
}

#[tauri::command]
pub async fn db_save_schedules_batch(
    state: State<'_, DbState>,
    items: Vec<ScheduleItem>,
) -> Result<Vec<i64>, String> {
    let ids = state.manager.add_schedules(&items)?;
    let change = DataChange {
        entity: "schedule".into(),
        action: "created".into(),
        id: ids.first().copied().unwrap_or(0),
        data: Some(serde_json::to_value(&items).unwrap()),
    };
    state.notify_change(change);
    Ok(ids)
}

#[tauri::command]
pub async fn db_delete_schedule(
    state: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    state.manager.delete_schedule(id)?;
    let change = DataChange {
        entity: "schedule".into(),
        action: "deleted".into(),
        id,
        data: None,
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_delete_schedules_by_date(
    state: State<'_, DbState>,
    date: String,
) -> Result<(), String> {
    state.manager.delete_schedules_by_date(&date)?;
    let change = DataChange {
        entity: "schedule".into(),
        action: "deleted".into(),
        id: 0,
        data: Some(serde_json::json!({ "date": date })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_toggle_schedule_status(
    state: State<'_, DbState>,
    id: i64,
    is_done: bool,
) -> Result<(), String> {
    state.manager.toggle_schedule_status(id, is_done)?;
    let change = DataChange {
        entity: "schedule".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "is_done": is_done })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_update_schedule_content(
    state: State<'_, DbState>,
    id: i64,
    content: String,
) -> Result<(), String> {
    let patch = SchedulePatch {
        content: Some(content.clone()),
        ..Default::default()
    };
    state.manager.patch_schedule(id, &patch)?;
    let change = DataChange {
        entity: "schedule".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "content": content })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_update_schedule_description(
    state: State<'_, DbState>,
    id: i64,
    description: Option<String>,
) -> Result<(), String> {
    let patch = SchedulePatch {
        description: description.clone(),
        ..Default::default()
    };
    state.manager.patch_schedule(id, &patch)?;
    let change = DataChange {
        entity: "schedule".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "description": description })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_update_schedule_date(
    state: State<'_, DbState>,
    id: i64,
    field: String,
    date: String,
) -> Result<(), String> {
    let patch = if field == "done_date" {
        SchedulePatch {
            done_date: Some(date.clone()),
            ..Default::default()
        }
    } else {
        SchedulePatch {
            create_date: Some(date.clone()),
            ..Default::default()
        }
    };
    state.manager.patch_schedule(id, &patch)?;
    let change = DataChange {
        entity: "schedule".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "field": field, "date": date })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_update_schedule_father_task(
    state: State<'_, DbState>,
    id: i64,
    father_task: Option<i64>,
) -> Result<(), String> {
    let patch = SchedulePatch {
        father_task,
        ..Default::default()
    };
    state.manager.patch_schedule(id, &patch)?;
    let change = DataChange {
        entity: "schedule".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "father_task": father_task })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_load_schedules_by_father_task(
    state: State<'_, DbState>,
    father_task_id: i64,
) -> Result<Vec<ScheduleItem>, String> {
    state.manager.get_schedules_by_father_task(father_task_id)
}

#[tauri::command]
pub async fn db_save_sub_task(
    state: State<'_, DbState>,
    content: String,
    father_task_id: i64,
    description: Option<String>,
) -> Result<i64, String> {
    let id = state.manager.save_sub_task(&content, father_task_id, description.as_deref())?;
    let change = DataChange {
        entity: "schedule".into(),
        action: "created".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "content": content, "father_task": father_task_id })),
    };
    state.notify_change(change);
    Ok(id)
}

#[tauri::command]
pub async fn db_search_schedules(
    state: State<'_, DbState>,
    keyword: String,
) -> Result<Vec<ScheduleItem>, String> {
    state.manager.search_schedules(&keyword, None)
}

// ========== MainTask Commands ==========

#[tauri::command]
pub async fn db_load_main_tasks(
    state: State<'_, DbState>,
) -> Result<Vec<MainTaskItem>, String> {
    state.manager.get_all_main_tasks(None)
}

#[tauri::command]
pub async fn db_search_main_tasks(
    state: State<'_, DbState>,
    keyword: String,
) -> Result<Vec<MainTaskItem>, String> {
    state.manager.search_main_tasks(&keyword, None)
}

#[tauri::command]
pub async fn db_save_main_task(
    state: State<'_, DbState>,
    content: String,
    description: Option<String>,
    priority: Option<i32>,
) -> Result<i64, String> {
    let item = MainTaskItem {
        id: 0,
        content,
        description,
        is_done: false,
        priority: priority.unwrap_or(0),
        create_date: String::new(),
        done_date: None,
    };
    let id = state.manager.add_main_task(&item)?;
    let change = DataChange {
        entity: "main_task".into(),
        action: "created".into(),
        id,
        data: Some(serde_json::to_value(&item).unwrap()),
    };
    state.notify_change(change);
    Ok(id)
}

#[tauri::command]
pub async fn db_update_main_task_content(
    state: State<'_, DbState>,
    id: i64,
    content: String,
) -> Result<(), String> {
    let patch = MainTaskPatch {
        content: Some(content.clone()),
        ..Default::default()
    };
    state.manager.patch_main_task(id, &patch)?;
    let change = DataChange {
        entity: "main_task".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "content": content })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_update_main_task_description(
    state: State<'_, DbState>,
    id: i64,
    description: Option<String>,
) -> Result<(), String> {
    let patch = MainTaskPatch {
        description: description.clone(),
        ..Default::default()
    };
    state.manager.patch_main_task(id, &patch)?;
    let change = DataChange {
        entity: "main_task".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "description": description })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_update_main_task_create_date(
    state: State<'_, DbState>,
    id: i64,
    create_date: String,
) -> Result<(), String> {
    let patch = MainTaskPatch {
        create_date: Some(create_date.clone()),
        ..Default::default()
    };
    state.manager.patch_main_task(id, &patch)?;
    let change = DataChange {
        entity: "main_task".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "create_date": create_date })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_update_main_task_done_date(
    state: State<'_, DbState>,
    id: i64,
    done_date: Option<String>,
) -> Result<(), String> {
    let patch = MainTaskPatch {
        done_date: done_date.clone(),
        ..Default::default()
    };
    state.manager.patch_main_task(id, &patch)?;
    let change = DataChange {
        entity: "main_task".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "done_date": done_date })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_toggle_main_task_status(
    state: State<'_, DbState>,
    id: i64,
    is_done: bool,
) -> Result<(), String> {
    state.manager.toggle_main_task_status(id, is_done)?;
    let change = DataChange {
        entity: "main_task".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "is_done": is_done })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_update_main_task_priority(
    state: State<'_, DbState>,
    id: i64,
    priority: i32,
) -> Result<(), String> {
    let patch = MainTaskPatch {
        priority: Some(priority),
        ..Default::default()
    };
    state.manager.patch_main_task(id, &patch)?;
    let change = DataChange {
        entity: "main_task".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "priority": priority })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_delete_main_task(
    state: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    state.manager.delete_main_task(id)?;
    let change = DataChange {
        entity: "main_task".into(),
        action: "deleted".into(),
        id,
        data: None,
    };
    state.notify_change(change);
    Ok(())
}

// ========== Note Commands ==========

#[tauri::command]
pub async fn db_load_notes(
    state: State<'_, DbState>,
) -> Result<Vec<NoteItem>, String> {
    state.manager.get_all_notes()
}

#[tauri::command]
pub async fn db_search_notes(
    state: State<'_, DbState>,
    keyword: String,
) -> Result<Vec<NoteItem>, String> {
    state.manager.search_notes(&keyword, None)
}

#[tauri::command]
pub async fn db_get_note(
    state: State<'_, DbState>,
    id: i64,
) -> Result<Option<NoteItem>, String> {
    state.manager.get_note(id)
}

#[tauri::command]
pub async fn db_create_note(
    state: State<'_, DbState>,
    title: Option<String>,
    content: Option<String>,
) -> Result<i64, String> {
    let item = NoteItem {
        id: 0,
        title: title.unwrap_or_default(),
        content: content.unwrap_or_default(),
        create_date: String::new(),
    };
    let id = state.manager.add_note(&item)?;
    let change = DataChange {
        entity: "note".into(),
        action: "created".into(),
        id,
        data: Some(serde_json::to_value(&item).unwrap()),
    };
    state.notify_change(change);
    Ok(id)
}

#[tauri::command]
pub async fn db_update_note(
    state: State<'_, DbState>,
    id: i64,
    title: String,
    content: String,
) -> Result<(), String> {
    let item = NoteItem {
        id,
        title,
        content,
        create_date: String::new(),
    };
    state.manager.update_note(id, &item)?;
    let change = DataChange {
        entity: "note".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::to_value(&item).unwrap()),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_update_note_title(
    state: State<'_, DbState>,
    id: i64,
    title: String,
) -> Result<(), String> {
    let patch = NotePatch {
        title: Some(title.clone()),
        ..Default::default()
    };
    state.manager.patch_note(id, &patch)?;
    let change = DataChange {
        entity: "note".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "title": title })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_update_note_content(
    state: State<'_, DbState>,
    id: i64,
    content: String,
) -> Result<(), String> {
    let patch = NotePatch {
        content: Some(content.clone()),
        ..Default::default()
    };
    state.manager.patch_note(id, &patch)?;
    let change = DataChange {
        entity: "note".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "content": content })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_update_note_create_date(
    state: State<'_, DbState>,
    id: i64,
    create_date: String,
) -> Result<(), String> {
    let patch = NotePatch {
        create_date: Some(create_date.clone()),
        ..Default::default()
    };
    state.manager.patch_note(id, &patch)?;
    let change = DataChange {
        entity: "note".into(),
        action: "updated".into(),
        id,
        data: Some(serde_json::json!({ "id": id, "create_date": create_date })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_delete_note(
    state: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    state.manager.delete_note(id)?;
    let change = DataChange {
        entity: "note".into(),
        action: "deleted".into(),
        id,
        data: None,
    };
    state.notify_change(change);
    Ok(())
}

// ========== Cell Color Commands ==========

#[tauri::command]
pub async fn db_update_cell_color(
    state: State<'_, DbState>,
    date: String,
    color: String,
) -> Result<(), String> {
    state.manager.update_cell_color(&date, &color)?;
    let change = DataChange {
        entity: "cell_color".into(),
        action: "updated".into(),
        id: 0,
        data: Some(serde_json::json!({ "date": date, "cell_color": color })),
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_get_cell_color(
    state: State<'_, DbState>,
    date: String,
) -> Result<String, String> {
    state.manager.get_cell_color(&date)
}

#[tauri::command]
pub async fn db_load_cell_colors(
    state: State<'_, DbState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<CellMetadata>, String> {
    state.manager.get_cell_colors_by_range(&start_date, &end_date)
}

// ========== Backup Commands ==========

#[tauri::command]
pub async fn db_export_all_data(
    state: State<'_, DbState>,
) -> Result<BackupData, String> {
    state.manager.export_all_data()
}

#[tauri::command]
pub async fn db_import_and_merge_data(
    state: State<'_, DbState>,
    data: BackupData,
) -> Result<ImportStats, String> {
    let stats = state.manager.import_and_merge_data(&data)?;
    let change = DataChange {
        entity: "batch".into(),
        action: "imported".into(),
        id: 0,
        data: Some(serde_json::to_value(&stats).unwrap()),
    };
    state.notify_change(change);
    Ok(stats)
}

#[tauri::command]
pub async fn db_clear_all_tables(
    state: State<'_, DbState>,
) -> Result<(), String> {
    state.manager.clear_all_tables()?;
    let change = DataChange {
        entity: "batch".into(),
        action: "cleared".into(),
        id: 0,
        data: None,
    };
    state.notify_change(change);
    Ok(())
}

#[tauri::command]
pub async fn db_reset_auto_increment(
    state: State<'_, DbState>,
) -> Result<(), String> {
    state.manager.reset_auto_increment()
}
