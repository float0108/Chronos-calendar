//! 窗口管理命令：设置、TaskBoard、Note 窗口

use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};

#[cfg(target_os = "windows")]
use crate::windows::apply_window_settings;

// === 窗口配置 ===

struct WindowConfig {
    name: &'static str,
    url: &'static str,
    title: &'static str,
    width: f64,
    height: f64,
    min_width: f64,
    min_height: f64,
}

impl WindowConfig {
    const fn new(
        name: &'static str,
        url: &'static str,
        title: &'static str,
        width: f64,
        height: f64,
        min_width: f64,
        min_height: f64,
    ) -> Self {
        Self {
            name,
            url,
            title,
            width,
            height,
            min_width,
            min_height,
        }
    }
}

// === 通用窗口创建辅助函数 ===

#[cfg(target_os = "windows")]
fn apply_window_effects(window: &tauri::WebviewWindow) -> Result<(), String> {
    if let Ok(tauri_hwnd) = window.hwnd() {
        unsafe {
            apply_window_settings(std::mem::transmute(tauri_hwnd));
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn apply_window_effects(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

fn build_window(
    app: &tauri::AppHandle,
    config: &WindowConfig,
    init_script: Option<String>,
) -> Result<tauri::WebviewWindow, String> {
    let mut builder = WebviewWindowBuilder::new(app, config.name, WebviewUrl::App(config.url.into()))
        .title(config.title)
        .inner_size(config.width, config.height)
        .min_inner_size(config.min_width, config.min_height)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .visible(false)
        .center();

    if let Some(script) = init_script {
        builder = builder.initialization_script(&script);
    }

    let window = builder.build().map_err(|e| e.to_string())?;

    let _ = window.set_skip_taskbar(true);
    apply_window_effects(&window)?;
    let _ = window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

    Ok(window)
}

fn show_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

fn toggle_window_by_config(app: &tauri::AppHandle, config: &WindowConfig) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window(config.name) {
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;
        if is_visible {
            let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
            window.hide().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            show_window(&window)?;
            Ok(true)
        }
    } else {
        let window = build_window(app, config, None)?;
        show_window(&window)?;
        Ok(true)
    }
}

fn open_window_by_config(app: &tauri::AppHandle, config: &WindowConfig) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(config.name) {
        show_window(&window)?;
        return Ok(());
    }

    let window = build_window(app, config, None)?;
    show_window(&window)?;
    Ok(())
}

fn close_window_by_name(app: &tauri::AppHandle, name: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(name) {
        let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn is_window_visible(app: &tauri::AppHandle, name: &str) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window(name) {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
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
        WebviewUrl::App("src/settings.html".into()),
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

// === TaskBoard 窗口 ===

const TASKBOARD_CONFIG: WindowConfig = WindowConfig::new(
    "taskboard",
    "src/taskboard.html",
    "Chronos - Tasks",
    640.0,
    520.0,
    500.0,
    400.0,
);

#[tauri::command]
pub async fn open_taskboard_window(app: tauri::AppHandle) -> Result<(), String> {
    open_window_by_config(&app, &TASKBOARD_CONFIG)
}

#[tauri::command]
pub async fn close_taskboard_window(app: tauri::AppHandle) -> Result<(), String> {
    close_window_by_name(&app, "taskboard")
}

#[tauri::command]
pub async fn toggle_taskboard_window(app: tauri::AppHandle) -> Result<bool, String> {
    toggle_window_by_config(&app, &TASKBOARD_CONFIG)
}

#[tauri::command]
pub async fn is_taskboard_window_visible(app: tauri::AppHandle) -> Result<bool, String> {
    is_window_visible(&app, "taskboard")
}

// === Note 窗口 ===

const NOTE_CONFIG: WindowConfig = WindowConfig::new(
    "note",
    "src/note.html",
    "Chronos - 备忘录",
    400.0,
    500.0,
    320.0,
    400.0,
);

#[tauri::command]
pub async fn open_note_window(app: tauri::AppHandle) -> Result<(), String> {
    open_window_by_config(&app, &NOTE_CONFIG)
}

#[tauri::command]
pub async fn close_note_window(app: tauri::AppHandle) -> Result<(), String> {
    close_window_by_name(&app, "note")
}

#[tauri::command]
pub async fn toggle_note_window(app: tauri::AppHandle) -> Result<bool, String> {
    toggle_window_by_config(&app, &NOTE_CONFIG)
}

#[tauri::command]
pub async fn is_note_window_visible(app: tauri::AppHandle) -> Result<bool, String> {
    is_window_visible(&app, "note")
}

// === Todo 窗口 ===

const TODO_CONFIG: WindowConfig = WindowConfig::new(
    "todo",
    "src/todo.html",
    "Chronos - Todo",
    320.0,
    480.0,
    280.0,
    360.0,
);

#[tauri::command]
pub async fn open_todo_window(app: tauri::AppHandle) -> Result<(), String> {
    open_window_by_config(&app, &TODO_CONFIG)
}

#[tauri::command]
pub async fn close_todo_window(app: tauri::AppHandle) -> Result<(), String> {
    close_window_by_name(&app, "todo")
}

#[tauri::command]
pub async fn toggle_todo_window(app: tauri::AppHandle) -> Result<bool, String> {
    toggle_window_by_config(&app, &TODO_CONFIG)
}

#[tauri::command]
pub async fn is_todo_window_visible(app: tauri::AppHandle) -> Result<bool, String> {
    is_window_visible(&app, "todo")
}
