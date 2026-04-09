//! 备忘录操作

use rusqlite::{params, OptionalExtension};
use super::manager::DatabaseManager;
use super::types::{NoteItem, NotePatch};

impl DatabaseManager {
    /// 添加备忘录
    pub fn add_note(&self, item: &NoteItem) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        let create_date = if item.create_date.is_empty() { &now } else { &item.create_date };

        conn.execute(
            "INSERT INTO notes (title, content, create_date) VALUES (?1, ?2, ?3)",
            params![item.title, item.content, create_date],
        ).map_err(|e| format!("Failed to add note: {}", e))?;

        Ok(conn.last_insert_rowid())
    }

    /// 获取单个备忘录
    pub fn get_note(&self, id: i64) -> Result<Option<NoteItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let result = conn.query_row(
            "SELECT id, title, content, create_date FROM notes WHERE id = ?1",
            params![id],
            |row| Ok(NoteItem {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                create_date: row.get(3)?,
            }),
        ).optional().map_err(|e| format!("Failed to get note: {}", e))?;
        Ok(result)
    }

    /// 获取所有备忘录
    pub fn get_all_notes(&self) -> Result<Vec<NoteItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content, create_date FROM notes ORDER BY id DESC"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map([], |row| {
            Ok(NoteItem {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                create_date: row.get(3)?,
            })
        }).map_err(|e| format!("Failed to get notes: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 搜索备忘录
    pub fn search_notes(&self, query: &str) -> Result<Vec<NoteItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let search_pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            "SELECT id, title, content, create_date FROM notes
             WHERE title LIKE ?1 OR content LIKE ?1
             ORDER BY id DESC"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![search_pattern], |row| {
            Ok(NoteItem {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                create_date: row.get(3)?,
            })
        }).map_err(|e| format!("Failed to search notes: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 更新备忘录
    pub fn update_note(&self, id: i64, item: &NoteItem) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, create_date = ?3 WHERE id = ?4",
            params![item.title, item.content, item.create_date, id],
        ).map_err(|e| format!("Failed to update note: {}", e))?;
        Ok(affected > 0)
    }

    /// 部分更新备忘录
    pub fn patch_note(&self, id: i64, updates: &NotePatch) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut sets = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref v) = updates.title {
            sets.push("title = ?");
            values.push(Box::new(v.clone()));
        }
        if let Some(ref v) = updates.content {
            sets.push("content = ?");
            values.push(Box::new(v.clone()));
        }
        if let Some(ref v) = updates.create_date {
            sets.push("create_date = ?");
            values.push(Box::new(v.clone()));
        }

        if sets.is_empty() {
            return Ok(false);
        }

        let sql = format!("UPDATE notes SET {} WHERE id = ?", sets.join(", "));
        values.push(Box::new(id));

        let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        let affected = conn.execute(&sql, params.as_slice())
            .map_err(|e| format!("Failed to patch note: {}", e))?;

        Ok(affected > 0)
    }

    /// 删除备忘录
    pub fn delete_note(&self, id: i64) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn.execute("DELETE FROM notes WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete note: {}", e))?;
        Ok(affected > 0)
    }
}
