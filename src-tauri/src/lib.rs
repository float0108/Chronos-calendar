mod commands;
mod db;
mod mcp;
mod tray;
mod windows;

use std::sync::Arc;
use tauri::{Manager, WindowEvent};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};
use tauri_plugin_autostart::MacosLauncher;
use crate::db::DatabaseManager;

#[cfg(target_os = "windows")]
use crate::windows::apply_window_settings;
#[cfg(target_os = "windows")]
use crate::windows::restore_window_proc;

#[cfg(target_os = "windows")]
use ::windows::Win32::Foundation::HWND;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .invoke_handler(tauri::generate_handler![
            // App commands
            commands::app::get_system_fonts,
            commands::app::set_autostart,
            commands::app::get_autostart_status,
            commands::app::start_mcp_server,
            commands::app::stop_mcp_server,
            commands::app::get_mcp_status,
            // Window commands
            commands::window::open_settings_window,
            commands::window::open_board_window,
            commands::window::close_board_window,
            commands::window::toggle_board_window,
            commands::window::is_board_window_visible,
            commands::window::open_note_window,
            commands::window::close_note_window,
            commands::window::toggle_note_window,
            commands::window::is_note_window_visible,
            commands::window::open_task_window,
            commands::window::close_task_window,
            commands::window::toggle_task_window,
            commands::window::is_task_window_visible,
            commands::window::open_search_window,
            commands::window::close_search_window,
            commands::window::toggle_search_window,
            commands::window::is_search_window_visible,
            // Database - Schedule commands
            commands::database::db_load_schedules,
            commands::database::db_load_todo_schedules,
            commands::database::db_load_done_schedules,
            commands::database::db_save_schedule,
            commands::database::db_delete_schedule,
            commands::database::db_delete_schedules_by_date,
            commands::database::db_toggle_schedule_status,
            commands::database::db_update_schedule_content,
            commands::database::db_update_schedule_description,
            commands::database::db_update_schedule_date,
            commands::database::db_update_schedule_father_task,
            commands::database::db_load_schedules_by_father_task,
            commands::database::db_save_sub_task,
            commands::database::db_search_schedules,
            // Database - MainTask commands
            commands::database::db_load_main_tasks,
            commands::database::db_search_main_tasks,
            commands::database::db_save_main_task,
            commands::database::db_update_main_task_content,
            commands::database::db_update_main_task_description,
            commands::database::db_update_main_task_create_date,
            commands::database::db_update_main_task_done_date,
            commands::database::db_toggle_main_task_status,
            commands::database::db_update_main_task_priority,
            commands::database::db_delete_main_task,
            // Database - Note commands
            commands::database::db_load_notes,
            commands::database::db_search_notes,
            commands::database::db_get_note,
            commands::database::db_create_note,
            commands::database::db_update_note,
            commands::database::db_update_note_title,
            commands::database::db_update_note_content,
            commands::database::db_update_note_create_date,
            commands::database::db_delete_note,
            // Database - Cell Color commands
            commands::database::db_update_cell_color,
            commands::database::db_get_cell_color,
            commands::database::db_load_cell_colors,
            // Database - Backup commands
            commands::database::db_export_all_data,
            commands::database::db_import_and_merge_data,
            commands::database::db_clear_all_tables,
            commands::database::db_reset_auto_increment
        ])
        .setup(|app| {
            // 初始化数据库管理器
            let db_manager = match DatabaseManager::new() {
                Ok(manager) => Arc::new(manager),
                Err(e) => {
                    eprintln!("Failed to initialize database: {}", e);
                    panic!("Database initialization failed: {}", e);
                }
            };

            // 管理 DbState
            app.manage(commands::DbState {
                manager: db_manager.clone(),
            });

            // 管理 MCP 服务状态
            app.manage(commands::McpState::default());

            // 自动启动 MCP 服务 (默认端口 3000)
            let mcp_state = app.state::<commands::McpState>();
            let handle = mcp_state.handle.clone();
            let app_handle = std::sync::Arc::new(app.handle().clone());

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut guard = handle.lock().await;
                    match crate::mcp::start_mcp_server(3000, Some(app_handle), Some(db_manager)) {
                        Ok(server_handle) => {
                            println!("MCP service started on port 3000");
                            *guard = Some(server_handle);
                        }
                        Err(e) => {
                            eprintln!("Failed to start MCP service: {}", e);
                        }
                    }
                });
            });

            let window = app.get_webview_window("main").unwrap();
            let _ = window.set_skip_taskbar(true);

            #[cfg(target_os = "windows")]
            {
                if let Ok(tauri_hwnd) = window.hwnd() {
                    unsafe {
                        let hwnd: HWND = std::mem::transmute(tauri_hwnd);
                        apply_window_settings(hwnd);
                    }
                }
            }

            // 创建系统托盘
            if let Err(e) = tray::setup_tray(app.handle()) {
                eprintln!("Failed to setup tray: {}", e);
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { .. } => {
                // 保存所有窗口状态
                let _ = window.app_handle().save_window_state(StateFlags::SIZE | StateFlags::POSITION);

                #[cfg(target_os = "windows")]
                {
                    if let Ok(tauri_hwnd) = window.hwnd() {
                        unsafe {
                            let hwnd: HWND = std::mem::transmute(tauri_hwnd);
                            restore_window_proc(hwnd);
                        }
                    }
                }
            }
            WindowEvent::Destroyed => {
                #[cfg(target_os = "windows")]
                {
                    if let Ok(tauri_hwnd) = window.hwnd() {
                        unsafe {
                            let hwnd: HWND = std::mem::transmute(tauri_hwnd);
                            restore_window_proc(hwnd);
                        }
                    }
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
