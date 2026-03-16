use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::image::Image;
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};
use font_kit::source::SystemSource;
use std::collections::HashSet;
use tauri_plugin_autostart::MacosLauncher;

// === Windows API 引入 ===
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    GetShellWindow, GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos,
    CallWindowProcW, GWLP_HWNDPARENT, GWL_EXSTYLE, GWL_WNDPROC, HWND_BOTTOM,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    WM_WINDOWPOSCHANGING, WINDOWPOS, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

// === 全局变量：存储各窗口的原始窗口过程 ===
#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::sync::{Mutex, LazyLock};
#[cfg(target_os = "windows")]
static OLD_WNDPROCS: LazyLock<Mutex<HashMap<isize, isize>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

// === 极简版窗口子类化：只负责死锁底层，不干涉焦点 ===
#[cfg(target_os = "windows")]
unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // 拦截系统改变层级，强行锁定在底层
    if msg == WM_WINDOWPOSCHANGING {
        let wp = &mut *(lparam.0 as *mut WINDOWPOS);
        if (wp.flags.0 & SWP_NOZORDER.0) == 0 {
            wp.hwndInsertAfter = HWND_BOTTOM;
        }
    }

    // 从 HashMap 中查找原始窗口过程
    let old_proc = OLD_WNDPROCS.lock().unwrap().get(&(hwnd.0 as isize)).copied();
    if let Some(old_proc) = old_proc {
        CallWindowProcW(std::mem::transmute(old_proc), hwnd, msg, wparam, lparam)
    } else {
        LRESULT(0)
    }
}

// === 应用窗口设置（隐藏任务栏、置底、子类化钩子）===
#[cfg(target_os = "windows")]
fn apply_window_settings(hwnd: HWND) {
    unsafe {
        // 1. 设置 TOOLWINDOW 隐藏任务栏图标
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let new_ex_style = (ex_style | WS_EX_TOOLWINDOW.0 as isize) & !(WS_EX_APPWINDOW.0 as isize);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style);

        // 2. 将桌面设为 Owner Window（让 Win+D 忽略最小化）
        let shell_hwnd = GetShellWindow();
        if !shell_hwnd.0.is_null() {
            SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, shell_hwnd.0 as isize);
        }

        // 3. 初始置底
        let _ = SetWindowPos(
            hwnd,
            HWND_BOTTOM,
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );

        // 4. 挂载 Z-Order 锁定钩子
        let prev_wndproc = SetWindowLongPtrW(
            hwnd,
            GWL_WNDPROC,
            subclass_proc as *const () as isize,
        );
        OLD_WNDPROCS.lock().unwrap().insert(hwnd.0 as isize, prev_wndproc);
    }
}

#[tauri::command]
fn get_system_fonts() -> Result<Vec<String>, String> {
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

#[tauri::command]
async fn set_autostart(app: tauri::AppHandle, enable: bool) -> Result<(), String> {
    let autostart_manager = app.state::<tauri_plugin_autostart::AutoLaunchManager>();
    if enable {
        autostart_manager.enable().map_err(|e| e.to_string())?;
    } else {
        autostart_manager.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn get_autostart_status(app: tauri::AppHandle) -> Result<bool, String> {
    let autostart_manager = app.state::<tauri_plugin_autostart::AutoLaunchManager>();
    autostart_manager.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
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
    .visible(false)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    let _ = settings_window.set_skip_taskbar(true);
    Ok(())
}

#[tauri::command]
async fn open_board_window(app: tauri::AppHandle) -> Result<(), String> {
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
    .visible(false)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    let _ = board_window.set_skip_taskbar(true);

    #[cfg(target_os = "windows")]
    {
        if let Ok(tauri_hwnd) = board_window.hwnd() {
            unsafe {
                let hwnd: HWND = std::mem::transmute(tauri_hwnd);
                apply_window_settings(hwnd);
            }
        }
    }

    // 恢复窗口状态
    let _ = board_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

    board_window.show().map_err(|e| e.to_string())?;
    board_window.set_focus().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn close_board_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("board") {
        // 保存窗口状态
        let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_board_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("board") {
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;
        if is_visible {
            // 保存窗口状态
            let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
            window.hide().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            Ok(true)
        }
    } else {
        // 窗口不存在，创建新窗口
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
        .visible(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

        let _ = board_window.set_skip_taskbar(true);

        #[cfg(target_os = "windows")]
        {
            if let Ok(tauri_hwnd) = board_window.hwnd() {
                unsafe {
                    let hwnd: HWND = std::mem::transmute(tauri_hwnd);
                    apply_window_settings(hwnd);
                }
            }
        }

        // 恢复窗口状态
        let _ = board_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

        board_window.show().map_err(|e| e.to_string())?;
        board_window.set_focus().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
async fn is_board_window_visible(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("board") {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

#[tauri::command]
async fn open_note_window(app: tauri::AppHandle) -> Result<(), String> {
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
    .visible(false)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    let _ = note_window.set_skip_taskbar(true);

    #[cfg(target_os = "windows")]
    {
        if let Ok(tauri_hwnd) = note_window.hwnd() {
            unsafe {
                let hwnd: HWND = std::mem::transmute(tauri_hwnd);
                apply_window_settings(hwnd);
            }
        }
    }

    // 恢复窗口状态
    let _ = note_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

    note_window.show().map_err(|e| e.to_string())?;
    note_window.set_focus().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn close_note_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("note") {
        // 保存窗口状态
        let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_note_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("note") {
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;
        if is_visible {
            // 保存窗口状态
            let _ = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION);
            window.hide().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            Ok(true)
        }
    } else {
        // 窗口不存在，创建新窗口
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
        .visible(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

        let _ = note_window.set_skip_taskbar(true);

        #[cfg(target_os = "windows")]
        {
            if let Ok(tauri_hwnd) = note_window.hwnd() {
                unsafe {
                    let hwnd: HWND = std::mem::transmute(tauri_hwnd);
                    apply_window_settings(hwnd);
                }
            }
        }

        // 恢复窗口状态
        let _ = note_window.restore_state(StateFlags::SIZE | StateFlags::POSITION);

        note_window.show().map_err(|e| e.to_string())?;
        note_window.set_focus().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
async fn is_note_window_visible(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("note") {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

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
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .invoke_handler(tauri::generate_handler![
            get_system_fonts,
            set_autostart,
            get_autostart_status,
            open_settings_window,
            open_board_window,
            close_board_window,
            toggle_board_window,
            is_board_window_visible,
            open_note_window,
            close_note_window,
            toggle_note_window,
            is_note_window_visible
        ])
        .setup(|app| {
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
            if let Err(e) = setup_tray(app.handle()) {
                eprintln!("Failed to setup tray: {}", e);
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            // 窗口关闭时保存状态
            WindowEvent::CloseRequested { .. } => {
                // 保存所有窗口状态
                let _ = window.app_handle().save_window_state(StateFlags::SIZE | StateFlags::POSITION);

                #[cfg(target_os = "windows")]
                {
                    if let Ok(tauri_hwnd) = window.hwnd() {
                        unsafe {
                            let hwnd: HWND = std::mem::transmute(tauri_hwnd);
                            if let Some(old_proc) = OLD_WNDPROCS.lock().unwrap().remove(&(hwnd.0 as isize)) {
                                SetWindowLongPtrW(hwnd, GWL_WNDPROC, old_proc);
                            }
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
                            if let Some(old_proc) = OLD_WNDPROCS.lock().unwrap().remove(&(hwnd.0 as isize)) {
                                SetWindowLongPtrW(hwnd, GWL_WNDPROC, old_proc);
                            }
                        }
                    }
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
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
        .on_menu_event(|app: &tauri::AppHandle, event| {
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