//! 单元格颜色操作

use rusqlite::{params, OptionalExtension};
use super::manager::DatabaseManager;
use super::types::{CellMetadata, TableStats};

impl DatabaseManager {
    /// 更新单元格颜色
    pub fn update_cell_color(&self, date: &str, color: &str) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn.execute(
            "INSERT INTO cell_metadata (date, cell_color) VALUES (?1, ?2)
             ON CONFLICT(date) DO UPDATE SET cell_color = ?2",
            params![date, color],
        ).map_err(|e| format!("Failed to update cell color: {}", e))?;
        Ok(affected > 0)
    }

    /// 获取单元格颜色
    pub fn get_cell_color(&self, date: &str) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let result = conn.query_row(
            "SELECT cell_color FROM cell_metadata WHERE date = ?1",
            params![date],
            |row| row.get::<_, String>(0),
        ).optional().map_err(|e| format!("Failed to get cell color: {}", e))?;
        Ok(result.unwrap_or_default())
    }

    /// 按日期范围加载单元格颜色
    pub fn get_cell_colors_by_range(&self, start_date: &str, end_date: &str) -> Result<Vec<CellMetadata>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT date, cell_color FROM cell_metadata WHERE date >= ?1 AND date <= ?2"
        ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let items = stmt.query_map(params![start_date, end_date], |row| {
            Ok(CellMetadata {
                date: row.get(0)?,
                cell_color: Some(row.get(1)?),
            })
        }).map_err(|e| format!("Failed to get cell colors: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect results: {}", e))?;

        Ok(items)
    }

    /// 批量导入单元格颜色
    pub fn import_cell_colors(&self, colors: &[CellMetadata]) -> Result<TableStats, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stats = TableStats::default();

        for item in colors {
            let existing = conn.query_row(
                "SELECT COUNT(*) FROM cell_metadata WHERE date = ?1",
                params![item.date],
                |row| row.get::<_, i32>(0),
            ).unwrap_or(0);

            conn.execute(
                "INSERT INTO cell_metadata (date, cell_color) VALUES (?1, ?2)
                 ON CONFLICT(date) DO UPDATE SET cell_color = ?2",
                params![item.date, item.cell_color.clone().unwrap_or_default()],
            ).map_err(|e| format!("Failed to import cell color: {}", e))?;

            if existing > 0 {
                stats.updated += 1;
            } else {
                stats.inserted += 1;
            }
        }

        Ok(stats)
    }
}
