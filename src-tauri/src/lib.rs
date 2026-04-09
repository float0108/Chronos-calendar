mod commands;
mod mcp;
mod tray;
mod windows;

use tauri::{Manager, WindowEvent};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};
use tauri_plugin_autostart::MacosLauncher;

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
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::get_system_fonts,
            commands::set_autostart,
            commands::get_autostart_status,
            commands::open_settings_window,
            commands::open_board_window,
            commands::close_board_window,
            commands::toggle_board_window,
            commands::is_board_window_visible,
            commands::open_note_window,
            commands::close_note_window,
            commands::toggle_note_window,
            commands::is_note_window_visible,
            commands::open_task_window,
            commands::close_task_window,
            commands::toggle_task_window,
            commands::is_task_window_visible,
            commands::open_search_window,
            commands::close_search_window,
            commands::toggle_search_window,
            commands::is_search_window_visible,
            commands::start_mcp_server,
            commands::stop_mcp_server,
            commands::get_mcp_status
        ])
        .setup(|app| {
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
                    match crate::mcp::start_mcp_server(3000, Some(app_handle)) {
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
