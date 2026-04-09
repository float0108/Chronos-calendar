use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};
use font_kit::source::SystemSource;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use crate::mcp::McpServerHandle;

#[cfg(target_os = "windows")]
use crate::windows::apply_window_settings;

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

// === 设置窗口 ===

#[tauri::command]
pub async fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let settings_window = WebviewWindowBuilder::new(
        &app,
        "settings",
        WebviewUrl::App("src/settings.html".into())
    )
    .title("Chronos - 设置")
    .inner_size(480.0, 600.0)
    .min_inner_size(480.0, 500.0)
    .resizable(true)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(false)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    let _ = settings_window.set_skip_taskbar(true);
    Ok(())
}

// === Board 窗口 ===

#[tauri::command]
pub async fn open_board_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("board") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let board_window = WebviewWindowBuilder::new(
        &app,
        "board",
        WebviewUrl::App("src/board.html".into())
    )
    .title("Chronos - Board")
    .inner_size(320.0, 480.0)
    .min_inner_size(280.0, 360.0)
    .resizable(true)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(false)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    let _ = board_window.set_skip_taskbar(true);

    #[cfg(target_os = "windows")]
    {
        if let Ok(tauri_hwnd) = board_window.hwnd() {
            unsafe {
                apply_window_settings(std::mem::transmute(tauri_hwnd));
            }
        }
    }

    let _ = board_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

    board_window.show().map_err(|e| e.to_string())?;
    board_window.set_focus().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn close_board_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("board") {
        let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_board_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("board") {
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;
        if is_visible {
            let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
            window.hide().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            Ok(true)
        }
    } else {
        let board_window = WebviewWindowBuilder::new(
            &app,
            "board",
            WebviewUrl::App("src/board.html".into())
        )
        .title("Chronos - Board")
        .inner_size(320.0, 480.0)
        .min_inner_size(280.0, 360.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .visible(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

        let _ = board_window.set_skip_taskbar(true);

        #[cfg(target_os = "windows")]
        {
            if let Ok(tauri_hwnd) = board_window.hwnd() {
                unsafe {
                    apply_window_settings(std::mem::transmute(tauri_hwnd));
                }
            }
        }

        let _ = board_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

        board_window.show().map_err(|e| e.to_string())?;
        board_window.set_focus().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub async fn is_board_window_visible(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("board") {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

// === Note 窗口 ===

#[tauri::command]
pub async fn open_note_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("note") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let note_window = WebviewWindowBuilder::new(
        &app,
        "note",
        WebviewUrl::App("src/note.html".into())
    )
    .title("Chronos - 备忘录")
    .inner_size(400.0, 500.0)
    .min_inner_size(320.0, 400.0)
    .resizable(true)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(false)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    let _ = note_window.set_skip_taskbar(true);

    #[cfg(target_os = "windows")]
    {
        if let Ok(tauri_hwnd) = note_window.hwnd() {
            unsafe {
                apply_window_settings(std::mem::transmute(tauri_hwnd));
            }
        }
    }

    let _ = note_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

    note_window.show().map_err(|e| e.to_string())?;
    note_window.set_focus().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn close_note_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("note") {
        let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_note_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("note") {
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;
        if is_visible {
            let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
            window.hide().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            Ok(true)
        }
    } else {
        let note_window = WebviewWindowBuilder::new(
            &app,
            "note",
            WebviewUrl::App("src/note.html".into())
        )
        .title("Chronos - 备忘录")
        .inner_size(400.0, 500.0)
        .min_inner_size(320.0, 400.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .visible(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

        let _ = note_window.set_skip_taskbar(true);

        #[cfg(target_os = "windows")]
        {
            if let Ok(tauri_hwnd) = note_window.hwnd() {
                unsafe {
                    apply_window_settings(std::mem::transmute(tauri_hwnd));
                }
            }
        }

        let _ = note_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

        note_window.show().map_err(|e| e.to_string())?;
        note_window.set_focus().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub async fn is_note_window_visible(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("note") {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

// === Task 窗口 ===

#[tauri::command]
pub async fn open_task_window(app: tauri::AppHandle, task_id: i64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("task") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        // 发送 task_id 给窗口
        window.emit("set_task_id", task_id).map_err(|e: tauri::Error| e.to_string())?;
        return Ok(());
    }

    let task_window = WebviewWindowBuilder::new(
        &app,
        "task",
        WebviewUrl::App("src/task.html".into())
    )
    .title("Chronos - Task")
    .inner_size(360.0, 480.0)
    .min_inner_size(300.0, 360.0)
    .resizable(true)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(false)
    .center()
    .initialization_script(format!("window.__TASK_ID__ = {};", task_id))
    .build()
    .map_err(|e| e.to_string())?;

    let _ = task_window.set_skip_taskbar(true);

    #[cfg(target_os = "windows")]
    {
        if let Ok(tauri_hwnd) = task_window.hwnd() {
            unsafe {
                apply_window_settings(std::mem::transmute(tauri_hwnd));
            }
        }
    }

    let _ = task_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

    task_window.show().map_err(|e| e.to_string())?;
    task_window.set_focus().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn close_task_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("task") {
        let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_task_window(app: tauri::AppHandle, task_id: Option<i64>) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("task") {
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;
        if is_visible {
            let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
            window.hide().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            if let Some(id) = task_id {
                window.emit("set_task_id", id).map_err(|e: tauri::Error| e.to_string())?;
            }
            Ok(true)
        }
    } else {
        let id = task_id.unwrap_or(0);
        let task_window = WebviewWindowBuilder::new(
            &app,
            "task",
            WebviewUrl::App("src/task.html".into())
        )
        .title("Chronos - Task")
        .inner_size(360.0, 480.0)
        .min_inner_size(300.0, 360.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .visible(false)
        .center()
        .initialization_script(format!("window.__TASK_ID__ = {};", id))
        .build()
        .map_err(|e| e.to_string())?;

        let _ = task_window.set_skip_taskbar(true);

        #[cfg(target_os = "windows")]
        {
            if let Ok(tauri_hwnd) = task_window.hwnd() {
                unsafe {
                    apply_window_settings(std::mem::transmute(tauri_hwnd));
                }
            }
        }

        let _ = task_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

        task_window.show().map_err(|e| e.to_string())?;
        task_window.set_focus().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub async fn is_task_window_visible(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("task") {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

// === Search 窗口 ===

#[tauri::command]
pub async fn open_search_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("search") {
        let _ = window.restore_state(StateFlags::SIZE | StateFlags::POSITION);
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let search_window = WebviewWindowBuilder::new(
        &app,
        "search",
        WebviewUrl::App("src/search.html".into())
    )
    .title("Chronos - 搜索")
    .inner_size(360.0, 480.0)
    .min_inner_size(300.0, 360.0)
    .resizable(true)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(false)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    let _ = search_window.set_skip_taskbar(true);

    #[cfg(target_os = "windows")]
    {
        if let Ok(tauri_hwnd) = search_window.hwnd() {
            unsafe {
                apply_window_settings(std::mem::transmute(tauri_hwnd));
            }
        }
    }

    let _ = search_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

    search_window.show().map_err(|e| e.to_string())?;
    search_window.set_focus().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn close_search_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("search") {
        let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_search_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("search") {
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;
        if is_visible {
            let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
            window.hide().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            Ok(true)
        }
    } else {
        let search_window = WebviewWindowBuilder::new(
            &app,
            "search",
            WebviewUrl::App("src/search.html".into())
        )
        .title("Chronos - 搜索")
        .inner_size(360.0, 480.0)
        .min_inner_size(300.0, 360.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .visible(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

        let _ = search_window.set_skip_taskbar(true);

        #[cfg(target_os = "windows")]
        {
            if let Ok(tauri_hwnd) = search_window.hwnd() {
                unsafe {
                    apply_window_settings(std::mem::transmute(tauri_hwnd));
                }
            }
        }

        let _ = search_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

        search_window.show().map_err(|e| e.to_string())?;
        search_window.set_focus().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub async fn is_search_window_visible(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("search") {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
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

    let port = port.unwrap_or(3000);
    let app_handle = std::sync::Arc::new(app.clone());
    let new_handle = crate::mcp::start_mcp_server(port, Some(app_handle))?;

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
