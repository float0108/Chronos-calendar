//! 应用级命令：系统字体、自启动、MCP 服务

use font_kit::source::SystemSource;
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use crate::mcp::McpServerHandle;

// === 系统字体 ===

#[tauri::command]
pub fn get_system_fonts() -> Result<Vec<String>, String> {
    let source = SystemSource::new();
    let all_fonts = source.all_fonts().map_err(|e| e.to_string())?;

    let mut font_names: HashSet<String> = HashSet::new();
    for handle in all_fonts {
        if let Ok(font) = handle.load() {
            font_names.insert(font.family_name());
        }
    }
    let mut fonts: Vec<String> = font_names.into_iter().collect();
    fonts.sort();
    Ok(fonts)
}

// === 开机自启动 ===

#[tauri::command]
pub async fn set_autostart(app: tauri::AppHandle, enable: bool) -> Result<(), String> {
    let autostart_manager = app.state::<tauri_plugin_autostart::AutoLaunchManager>();
    if enable {
        autostart_manager.enable().map_err(|e| e.to_string())?;
    } else {
        autostart_manager.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_autostart_status(app: tauri::AppHandle) -> Result<bool, String> {
    let autostart_manager = app.state::<tauri_plugin_autostart::AutoLaunchManager>();
    autostart_manager.is_enabled().map_err(|e| e.to_string())
}

// === MCP 服务 ===

/// MCP 服务状态
pub struct McpState {
    pub handle: Arc<Mutex<Option<McpServerHandle>>>,
}

impl Default for McpState {
    fn default() -> Self {
        Self {
            handle: Arc::new(Mutex::new(None)),
        }
    }
}

/// 启动 MCP 服务器
#[tauri::command]
pub async fn start_mcp_server(
    app: tauri::AppHandle,
    port: Option<u16>,
) -> Result<String, String> {
    let state = app.state::<McpState>();
    let mut handle_guard = state.handle.lock().await;

    // 如果已经启动，返回当前状态
    if handle_guard.is_some() {
        return Ok("MCP 服务已在运行中".to_string());
    }

    // 获取共享的 DatabaseManager
    let db_state = app.state::<super::database::DbState>();
    let db_manager = db_state.manager.clone();

    let port = port.unwrap_or(3000);
    let app_handle = Arc::new(app.clone());
    let new_handle = crate::mcp::start_mcp_server(port, Some(app_handle), Some(db_manager))?;

    *handle_guard = Some(new_handle);

    Ok(format!("MCP 服务已启动，监听端口 {}", port))
}

/// 停止 MCP 服务器
#[tauri::command]
pub async fn stop_mcp_server(app: tauri::AppHandle) -> Result<String, String> {
    let state = app.state::<McpState>();
    let mut handle_guard = state.handle.lock().await;

    if let Some(handle) = handle_guard.take() {
        handle.cancel_token.cancel();
        Ok("MCP 服务已停止".to_string())
    } else {
        Ok("MCP 服务未在运行".to_string())
    }
}

/// 获取 MCP 服务状态
#[tauri::command]
pub async fn get_mcp_status(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let state = app.state::<McpState>();
    let handle_guard = state.handle.lock().await;

    if let Some(handle) = handle_guard.as_ref() {
        Ok(json!({
            "running": true,
            "port": handle.addr.port()
        }))
    } else {
        Ok(json!({
            "running": false,
            "port": null
        }))
    }
}
