//! 日程操作

use rusqlite::{params, OptionalExtension};
use super::manager::DatabaseManager;
use super::types::{ScheduleItem, SchedulePatch};

impl DatabaseManager {
    // ========== 基础 CRUD ==========

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

    /// 按日期删除日程
    pub fn delete_schedules_by_date(&self, date: &str) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn.execute("DELETE FROM schedules WHERE create_date = ?1", params![date])
            .map_err(|e| format!("Failed to delete schedules by date: {}", e))?;
        Ok(affected)
    }

    // ========== 查询 ==========

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

    /// 按日期范围加载日程
    pub fn get_schedules_by_range(&self, start_date: &str, end_date: &str) -> Result<Vec<ScheduleItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, create_date, content, is_done, priority, done_date, description, father_task
             FROM schedules
             WHERE create_date >= ?1 AND create_date <= ?2
             ORDER BY id ASC"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![start_date, end_date], |row| {
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

    /// 按日期范围加载待办日程（未完成的日程）
    pub fn get_todo_schedules_by_range(&self, start_date: &str, end_date: &str) -> Result<Vec<ScheduleItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, create_date, content, is_done, priority, done_date, description, father_task
             FROM schedules
             WHERE create_date >= ?1 AND create_date <= ?2 AND is_done = 0
             ORDER BY id ASC"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![start_date, end_date], |row| {
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
        }).map_err(|e| format!("Failed to get todo schedules: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 按完成日期范围加载日程
    pub fn get_done_schedules_by_range(&self, start_date: &str, end_date: &str) -> Result<Vec<ScheduleItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, create_date, content, is_done, priority, done_date, description, father_task
             FROM schedules
             WHERE done_date >= ?1 AND done_date <= ?2 AND is_done = 1
             ORDER BY done_date, id ASC"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![start_date, end_date], |row| {
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
        }).map_err(|e| format!("Failed to get done schedules: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 按主任务ID加载子任务
    pub fn get_schedules_by_father_task(&self, father_task_id: i64) -> Result<Vec<ScheduleItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, create_date, content, is_done, priority, done_date, description, father_task
             FROM schedules
             WHERE father_task = ?1
             ORDER BY is_done ASC, create_date DESC, id DESC"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![father_task_id], |row| {
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
        }).map_err(|e| format!("Failed to get schedules by father task: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
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

    // ========== 状态切换 ==========

    /// 切换日程状态（自动设置done_date）
    pub fn toggle_schedule_status(&self, id: i64, is_done: bool) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        if is_done {
            let now = chrono::Local::now().format("%Y-%m-%d").to_string();
            let affected = conn.execute(
                "UPDATE schedules SET is_done = 1, done_date = ?1 WHERE id = ?2",
                params![now, id],
            ).map_err(|e| format!("Failed to toggle schedule status: {}", e))?;
            Ok(affected > 0)
        } else {
            let affected = conn.execute(
                "UPDATE schedules SET is_done = 0, done_date = NULL WHERE id = ?1",
                params![id],
            ).map_err(|e| format!("Failed to toggle schedule status: {}", e))?;
            Ok(affected > 0)
        }
    }

    /// 保存子任务（日程，create_date 可为空）
    pub fn save_sub_task(&self, content: &str, father_task_id: i64, description: Option<&str>) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO schedules (create_date, content, is_done, priority, description, father_task)
             VALUES (NULL, ?1, 0, 0, ?2, ?3)",
            params![content, description, father_task_id],
        ).map_err(|e| format!("Failed to save sub task: {}", e))?;
        Ok(conn.last_insert_rowid())
    }
}
