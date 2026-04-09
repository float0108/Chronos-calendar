//! 备份恢复操作

use rusqlite::params;
use super::manager::DatabaseManager;
use super::types::{
    BackupData, ImportStats, TableStats,
    ScheduleItem, MainTaskItem, NoteItem, CellMetadata,
};

impl DatabaseManager {
    /// 导出所有数据
    pub fn export_all_data(&self) -> Result<BackupData, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // 导出 schedules
        let mut stmt = conn.prepare(
            "SELECT id, create_date, content, is_done, priority, done_date, description, father_task FROM schedules ORDER BY id"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;
        let schedules = stmt.query_map([], |row| {
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
        }).map_err(|e| format!("Failed to export schedules: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        // 导出 main_tasks
        let mut stmt = conn.prepare(
            "SELECT id, content, description, is_done, priority, create_date, done_date FROM main_tasks ORDER BY id"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;
        let main_tasks = stmt.query_map([], |row| {
            Ok(MainTaskItem {
                id: row.get(0)?,
                content: row.get(1)?,
                description: row.get(2)?,
                is_done: row.get::<_, i32>(3)? != 0,
                priority: row.get(4)?,
                create_date: row.get(5)?,
                done_date: row.get(6)?,
            })
        }).map_err(|e| format!("Failed to export main tasks: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        // 导出 notes
        let mut stmt = conn.prepare(
            "SELECT id, title, content, create_date FROM notes ORDER BY id"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;
        let notes = stmt.query_map([], |row| {
            Ok(NoteItem {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                create_date: row.get(3)?,
            })
        }).map_err(|e| format!("Failed to export notes: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        // 导出 cell_metadata
        let mut stmt = conn.prepare(
            "SELECT date, cell_color FROM cell_metadata WHERE cell_color != ''"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;
        let cell_metadata = stmt.query_map([], |row| {
            Ok(CellMetadata {
                date: row.get(0)?,
                cell_color: Some(row.get(1)?),
            })
        }).map_err(|e| format!("Failed to export cell metadata: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(BackupData {
            schedules,
            main_tasks,
            notes,
            cell_metadata,
        })
    }

    /// 导入合并数据
    pub fn import_and_merge_data(&self, data: &BackupData) -> Result<ImportStats, String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stats = ImportStats::default();

        let tx = conn.transaction().map_err(|e| format!("Failed to start transaction: {}", e))?;

        // 导入 schedules
        for item in &data.schedules {
            let existing = tx.query_row(
                "SELECT COUNT(*) FROM schedules WHERE id = ?1",
                params![item.id],
                |row| row.get::<_, i32>(0),
            ).unwrap_or(0);

            if existing > 0 {
                tx.execute(
                    "UPDATE schedules SET create_date = ?1, content = ?2, is_done = ?3, priority = ?4,
                     done_date = ?5, description = ?6, father_task = ?7 WHERE id = ?8",
                    params![
                        item.create_date, item.content, item.is_done as i32, item.priority,
                        item.done_date, item.description, item.father_task, item.id
                    ],
                ).map_err(|e| format!("Failed to update schedule: {}", e))?;
                stats.schedules.updated += 1;
            } else {
                tx.execute(
                    "INSERT INTO schedules (id, create_date, content, is_done, priority, done_date, description, father_task)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        item.id, item.create_date, item.content, item.is_done as i32, item.priority,
                        item.done_date, item.description, item.father_task
                    ],
                ).map_err(|e| format!("Failed to insert schedule: {}", e))?;
                stats.schedules.inserted += 1;
            }
        }

        // 导入 main_tasks
        for item in &data.main_tasks {
            let existing = tx.query_row(
                "SELECT COUNT(*) FROM main_tasks WHERE id = ?1",
                params![item.id],
                |row| row.get::<_, i32>(0),
            ).unwrap_or(0);

            if existing > 0 {
                tx.execute(
                    "UPDATE main_tasks SET content = ?1, description = ?2, is_done = ?3, priority = ?4,
                     create_date = ?5, done_date = ?6 WHERE id = ?7",
                    params![
                        item.content, item.description, item.is_done as i32, item.priority,
                        item.create_date, item.done_date, item.id
                    ],
                ).map_err(|e| format!("Failed to update main task: {}", e))?;
                stats.main_tasks.updated += 1;
            } else {
                tx.execute(
                    "INSERT INTO main_tasks (id, content, description, is_done, priority, create_date, done_date)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        item.id, item.content, item.description, item.is_done as i32, item.priority,
                        item.create_date, item.done_date
                    ],
                ).map_err(|e| format!("Failed to insert main task: {}", e))?;
                stats.main_tasks.inserted += 1;
            }
        }

        // 导入 notes
        for item in &data.notes {
            let existing = tx.query_row(
                "SELECT COUNT(*) FROM notes WHERE id = ?1",
                params![item.id],
                |row| row.get::<_, i32>(0),
            ).unwrap_or(0);

            if existing > 0 {
                tx.execute(
                    "UPDATE notes SET title = ?1, content = ?2, create_date = ?3 WHERE id = ?4",
                    params![item.title, item.content, item.create_date, item.id],
                ).map_err(|e| format!("Failed to update note: {}", e))?;
                stats.notes.updated += 1;
            } else {
                tx.execute(
                    "INSERT INTO notes (id, title, content, create_date) VALUES (?1, ?2, ?3, ?4)",
                    params![item.id, item.title, item.content, item.create_date],
                ).map_err(|e| format!("Failed to insert note: {}", e))?;
                stats.notes.inserted += 1;
            }
        }

        // 导入 cell_metadata
        for item in &data.cell_metadata {
            let existing = tx.query_row(
                "SELECT COUNT(*) FROM cell_metadata WHERE date = ?1",
                params![item.date],
                |row| row.get::<_, i32>(0),
            ).unwrap_or(0);

            tx.execute(
                "INSERT INTO cell_metadata (date, cell_color) VALUES (?1, ?2)
                 ON CONFLICT(date) DO UPDATE SET cell_color = ?2",
                params![item.date, item.cell_color.clone().unwrap_or_default()],
            ).map_err(|e| format!("Failed to import cell metadata: {}", e))?;

            if existing > 0 {
                stats.cell_metadata.updated += 1;
            } else {
                stats.cell_metadata.inserted += 1;
            }
        }

        tx.commit().map_err(|e| format!("Failed to commit transaction: {}", e))?;
        Ok(stats)
    }

    /// 清空所有表
    pub fn clear_all_tables(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(
            "DELETE FROM schedules;
             DELETE FROM main_tasks;
             DELETE FROM notes;
             DELETE FROM cell_metadata;"
        ).map_err(|e| format!("Failed to clear all tables: {}", e))?;
        Ok(())
    }

    /// 重置自增ID
    pub fn reset_auto_increment(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM sqlite_sequence WHERE name IN ('schedules', 'main_tasks', 'notes')",
            [],
        ).map_err(|e| format!("Failed to reset auto increment: {}", e))?;
        Ok(())
    }
}
