use tauri::{Manager, AppHandle};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::image::Image;
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 创建菜单项
    let show_calendar = MenuItem::with_id(app, "show_calendar", "Calendar", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_calendar, &exit])?;

    // 尝试加载图标，失败则使用默认图标
    let icon = app.path().resource_dir()
        .ok()
        .and_then(|dir| {
            let icon_path = dir.join("icons").join("icon.ico");
            Image::from_path(&icon_path).ok()
        })
        .or_else(|| app.default_window_icon().cloned())
        .ok_or("Failed to load tray icon")?;

    // 创建托盘图标
    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app: &AppHandle, event| {
            match event.id.as_ref() {
                "show_calendar" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
                "exit" => {
                    // 退出前保存所有窗口状态
                    let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
