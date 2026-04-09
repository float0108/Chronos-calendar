//! 数据库 Tauri 命令
//!
//! 提供前端调用的数据库操作接口

use std::sync::Arc;
use tauri::State;

use crate::db::*;

/// 数据库状态
pub struct DbState {
    pub manager: Arc<DatabaseManager>,
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
    state.manager.get_schedules_by_range(&start_date, &end_date)
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
    state.manager.add_schedule(&item)
}

#[tauri::command]
pub async fn db_delete_schedule(
    state: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    state.manager.delete_schedule(id)?;
    Ok(())
}

#[tauri::command]
pub async fn db_delete_schedules_by_date(
    state: State<'_, DbState>,
    date: String,
) -> Result<(), String> {
    state.manager.delete_schedules_by_date(&date)?;
    Ok(())
}

#[tauri::command]
pub async fn db_toggle_schedule_status(
    state: State<'_, DbState>,
    id: i64,
    is_done: bool,
) -> Result<(), String> {
    state.manager.toggle_schedule_status(id, is_done)?;
    Ok(())
}

#[tauri::command]
pub async fn db_update_schedule_content(
    state: State<'_, DbState>,
    id: i64,
    content: String,
) -> Result<(), String> {
    let patch = SchedulePatch {
        content: Some(content),
        ..Default::default()
    };
    state.manager.patch_schedule(id, &patch)?;
    Ok(())
}

#[tauri::command]
pub async fn db_update_schedule_description(
    state: State<'_, DbState>,
    id: i64,
    description: Option<String>,
) -> Result<(), String> {
    let patch = SchedulePatch {
        description,
        ..Default::default()
    };
    state.manager.patch_schedule(id, &patch)?;
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
            done_date: Some(date),
            ..Default::default()
        }
    } else {
        SchedulePatch {
            create_date: Some(date),
            ..Default::default()
        }
    };
    state.manager.patch_schedule(id, &patch)?;
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
    state.manager.save_sub_task(&content, father_task_id, description.as_deref())
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
    state.manager.add_main_task(&item)
}

#[tauri::command]
pub async fn db_update_main_task_content(
    state: State<'_, DbState>,
    id: i64,
    content: String,
) -> Result<(), String> {
    let patch = MainTaskPatch {
        content: Some(content),
        ..Default::default()
    };
    state.manager.patch_main_task(id, &patch)?;
    Ok(())
}

#[tauri::command]
pub async fn db_update_main_task_description(
    state: State<'_, DbState>,
    id: i64,
    description: Option<String>,
) -> Result<(), String> {
    let patch = MainTaskPatch {
        description,
        ..Default::default()
    };
    state.manager.patch_main_task(id, &patch)?;
    Ok(())
}

#[tauri::command]
pub async fn db_update_main_task_create_date(
    state: State<'_, DbState>,
    id: i64,
    create_date: String,
) -> Result<(), String> {
    let patch = MainTaskPatch {
        create_date: Some(create_date),
        ..Default::default()
    };
    state.manager.patch_main_task(id, &patch)?;
    Ok(())
}

#[tauri::command]
pub async fn db_update_main_task_done_date(
    state: State<'_, DbState>,
    id: i64,
    done_date: Option<String>,
) -> Result<(), String> {
    let patch = MainTaskPatch {
        done_date,
        ..Default::default()
    };
    state.manager.patch_main_task(id, &patch)?;
    Ok(())
}

#[tauri::command]
pub async fn db_toggle_main_task_status(
    state: State<'_, DbState>,
    id: i64,
    is_done: bool,
) -> Result<(), String> {
    state.manager.toggle_main_task_status(id, is_done)?;
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
    Ok(())
}

#[tauri::command]
pub async fn db_delete_main_task(
    state: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    state.manager.delete_main_task(id)?;
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
    state.manager.search_notes(&keyword)
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
    state.manager.add_note(&item)
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
    Ok(())
}

#[tauri::command]
pub async fn db_update_note_title(
    state: State<'_, DbState>,
    id: i64,
    title: String,
) -> Result<(), String> {
    let patch = NotePatch {
        title: Some(title),
        ..Default::default()
    };
    state.manager.patch_note(id, &patch)?;
    Ok(())
}

#[tauri::command]
pub async fn db_update_note_content(
    state: State<'_, DbState>,
    id: i64,
    content: String,
) -> Result<(), String> {
    let patch = NotePatch {
        content: Some(content),
        ..Default::default()
    };
    state.manager.patch_note(id, &patch)?;
    Ok(())
}

#[tauri::command]
pub async fn db_update_note_create_date(
    state: State<'_, DbState>,
    id: i64,
    create_date: String,
) -> Result<(), String> {
    let patch = NotePatch {
        create_date: Some(create_date),
        ..Default::default()
    };
    state.manager.patch_note(id, &patch)?;
    Ok(())
}

#[tauri::command]
pub async fn db_delete_note(
    state: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    state.manager.delete_note(id)?;
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
    state.manager.import_and_merge_data(&data)
}

#[tauri::command]
pub async fn db_clear_all_tables(
    state: State<'_, DbState>,
) -> Result<(), String> {
    state.manager.clear_all_tables()
}

#[tauri::command]
pub async fn db_reset_auto_increment(
    state: State<'_, DbState>,
) -> Result<(), String> {
    state.manager.reset_auto_increment()
}
