//! 主任务操作

use rusqlite::{params, OptionalExtension};
use super::manager::DatabaseManager;
use super::types::{MainTaskItem, MainTaskPatch};

impl DatabaseManager {
    /// 添加主任务
    pub fn add_main_task(&self, item: &MainTaskItem) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        let create_date = if item.create_date.is_empty() { &now } else { &item.create_date };

        conn.execute(
            "INSERT INTO main_tasks (content, description, is_done, priority, create_date, done_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.content,
                item.description,
                item.is_done as i32,
                item.priority,
                create_date,
                item.done_date
            ],
        ).map_err(|e| format!("Failed to add main task: {}", e))?;

        Ok(conn.last_insert_rowid())
    }

    /// 批量添加主任务
    pub fn add_main_tasks(&self, items: &[MainTaskItem]) -> Result<Vec<i64>, String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut ids = Vec::new();
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();

        let tx = conn.transaction().map_err(|e| format!("Failed to start transaction: {}", e))?;

        for item in items {
            let create_date = if item.create_date.is_empty() { &now } else { &item.create_date };
            tx.execute(
                "INSERT INTO main_tasks (content, description, is_done, priority, create_date, done_date)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    item.content,
                    item.description,
                    item.is_done as i32,
                    item.priority,
                    create_date,
                    item.done_date
                ],
            ).map_err(|e| format!("Failed to add main task: {}", e))?;
            ids.push(tx.last_insert_rowid());
        }

        tx.commit().map_err(|e| format!("Failed to commit transaction: {}", e))?;
        Ok(ids)
    }

    /// 更新主任务
    pub fn update_main_task(&self, id: i64, item: &MainTaskItem) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn.execute(
            "UPDATE main_tasks SET content = ?1, description = ?2, is_done = ?3, priority = ?4,
             create_date = ?5, done_date = ?6 WHERE id = ?7",
            params![
                item.content,
                item.description,
                item.is_done as i32,
                item.priority,
                item.create_date,
                item.done_date,
                id
            ],
        ).map_err(|e| format!("Failed to update main task: {}", e))?;

        Ok(affected > 0)
    }

    /// 部分更新主任务
    pub fn patch_main_task(&self, id: i64, updates: &MainTaskPatch) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut sets = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref v) = updates.content {
            sets.push("content = ?");
            values.push(Box::new(v.clone()));
        }
        if updates.description.is_some() {
            sets.push("description = ?");
            values.push(Box::new(updates.description.clone()));
        }
        if let Some(v) = updates.is_done {
            sets.push("is_done = ?");
            values.push(Box::new(v as i32));
        }
        if let Some(v) = updates.priority {
            sets.push("priority = ?");
            values.push(Box::new(v));
        }
        if let Some(ref v) = updates.create_date {
            sets.push("create_date = ?");
            values.push(Box::new(v.clone()));
        }
        if updates.done_date.is_some() {
            sets.push("done_date = ?");
            values.push(Box::new(updates.done_date.clone()));
        }

        if sets.is_empty() {
            return Ok(false);
        }

        let sql = format!("UPDATE main_tasks SET {} WHERE id = ?", sets.join(", "));
        values.push(Box::new(id));

        let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        let affected = conn.execute(&sql, params.as_slice())
            .map_err(|e| format!("Failed to patch main task: {}", e))?;

        Ok(affected > 0)
    }

    /// 删除主任务
    pub fn delete_main_task(&self, id: i64) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn.execute("DELETE FROM main_tasks WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete main task: {}", e))?;
        Ok(affected > 0)
    }

    /// 批量删除主任务
    pub fn delete_main_tasks(&self, ids: &[i64]) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!("DELETE FROM main_tasks WHERE id IN ({})", placeholders.join(","));
        let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let affected = conn.execute(&sql, params.as_slice())
            .map_err(|e| format!("Failed to delete main tasks: {}", e))?;
        Ok(affected)
    }

    /// 获取单个主任务
    pub fn get_main_task(&self, id: i64) -> Result<Option<MainTaskItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let result = conn.query_row(
            "SELECT id, content, description, is_done, priority, create_date, done_date
             FROM main_tasks WHERE id = ?1",
            params![id],
            |row| Ok(MainTaskItem {
                id: row.get(0)?,
                content: row.get(1)?,
                description: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                create_date: row.get(5)?,
                done_date: row.get(6)?,
            }),
        ).optional().map_err(|e| format!("Failed to get main task: {}", e))?;
        Ok(result)
    }

    /// 搜索主任务
    pub fn search_main_tasks(&self, query: &str, limit: Option<usize>) -> Result<Vec<MainTaskItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let limit = limit.unwrap_or(100);
        let search_pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            "SELECT id, content, description, is_done, priority, create_date, done_date
             FROM main_tasks
             WHERE content LIKE ?1 OR description LIKE ?1
             ORDER BY is_done ASC, create_date DESC, id DESC
             LIMIT ?2"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![search_pattern, limit as i32], |row| {
            Ok(MainTaskItem {
                id: row.get(0)?,
                content: row.get(1)?,
                description: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                create_date: row.get(5)?,
                done_date: row.get(6)?,
            })
        }).map_err(|e| format!("Failed to search main tasks: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 获取所有主任务（带限制）
    pub fn get_all_main_tasks(&self, limit: Option<usize>) -> Result<Vec<MainTaskItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let limit = limit.unwrap_or(100);

        let mut stmt = conn.prepare(
            "SELECT id, content, description, is_done, priority, create_date, done_date
             FROM main_tasks
             ORDER BY is_done ASC, create_date DESC, id DESC
             LIMIT ?1"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![limit as i32], |row| {
            Ok(MainTaskItem {
                id: row.get(0)?,
                content: row.get(1)?,
                description: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                create_date: row.get(5)?,
                done_date: row.get(6)?,
            })
        }).map_err(|e| format!("Failed to get main tasks: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 获取进行中的主任务（未完成的）
    pub fn get_active_main_tasks(&self, limit: Option<usize>) -> Result<Vec<MainTaskItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let limit = limit.unwrap_or(100);

        let mut stmt = conn.prepare(
            "SELECT id, content, description, is_done, priority, create_date, done_date
             FROM main_tasks
             WHERE is_done = 0
             ORDER BY priority DESC, create_date ASC
             LIMIT ?1"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![limit as i32], |row| {
            Ok(MainTaskItem {
                id: row.get(0)?,
                content: row.get(1)?,
                description: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                create_date: row.get(5)?,
                done_date: row.get(6)?,
            })
        }).map_err(|e| format!("Failed to get active main tasks: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 切换主任务状态（自动设置done_date）
    pub fn toggle_main_task_status(&self, id: i64, is_done: bool) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        if is_done {
            let now = chrono::Local::now().format("%Y-%m-%d").to_string();
            let affected = conn.execute(
                "UPDATE main_tasks SET is_done = 1, done_date = ?1 WHERE id = ?2",
                params![now, id],
            ).map_err(|e| format!("Failed to toggle main task status: {}", e))?;
            Ok(affected > 0)
        } else {
            let affected = conn.execute(
                "UPDATE main_tasks SET is_done = 0, done_date = NULL WHERE id = ?1",
                params![id],
            ).map_err(|e| format!("Failed to toggle main task status: {}", e))?;
            Ok(affected > 0)
        }
    }
}
