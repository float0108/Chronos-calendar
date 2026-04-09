//! 统计操作

use rusqlite::params;
use chrono::Datelike;
use super::manager::DatabaseManager;
use super::types::{TodayStats, WeekStats};

impl DatabaseManager {
    /// 获取今日日程统计
    pub fn get_today_stats(&self) -> Result<TodayStats, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        let total: i32 = conn.query_row(
            "SELECT COUNT(*) FROM schedules WHERE create_date = ?1",
            params![today],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count schedules: {}", e))?;

        let completed: i32 = conn.query_row(
            "SELECT COUNT(*) FROM schedules WHERE create_date = ?1 AND is_done = 1",
            params![today],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count completed schedules: {}", e))?;

        Ok(TodayStats { total, completed })
    }

    /// 获取本周日程统计
    pub fn get_week_stats(&self) -> Result<WeekStats, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let today = chrono::Local::now();
        let week_start = (today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64))
            .format("%Y-%m-%d").to_string();
        let week_end = (today + chrono::Duration::days(6 - today.weekday().num_days_from_monday() as i64))
            .format("%Y-%m-%d").to_string();

        let total: i32 = conn.query_row(
            "SELECT COUNT(*) FROM schedules WHERE create_date >= ?1 AND create_date <= ?2",
            params![week_start, week_end],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count schedules: {}", e))?;

        let completed: i32 = conn.query_row(
            "SELECT COUNT(*) FROM schedules WHERE create_date >= ?1 AND create_date <= ?2 AND is_done = 1",
            params![week_start, week_end],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count completed schedules: {}", e))?;

        Ok(WeekStats { total, completed, week_start, week_end })
    }
}
