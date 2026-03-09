use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
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

// === 全局变量：存储原始窗口过程 ===
#[cfg(target_os = "windows")]
static mut OLD_WNDPROC: Option<isize> = None;

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

    if let Some(old_proc) = OLD_WNDPROC {
        CallWindowProcW(std::mem::transmute(old_proc), hwnd, msg, wparam, lparam)
    } else {
        LRESULT(0)
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
        .invoke_handler(tauri::generate_handler![get_system_fonts, set_autostart, get_autostart_status, open_settings_window])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let _ = window.set_skip_taskbar(true);

            #[cfg(target_os = "windows")]
            {
                if let Ok(tauri_hwnd) = window.hwnd() {
                    unsafe {
                        let hwnd: HWND = std::mem::transmute(tauri_hwnd);

                        // 1. 设置 TOOLWINDOW 隐藏任务栏图标。
                        // 【关键】：放弃 WS_EX_NOACTIVATE，让窗口正常拥有焦点周期
                        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                        let new_ex_style = (ex_style | WS_EX_TOOLWINDOW.0 as isize) & !(WS_EX_APPWINDOW.0 as isize);
                        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style);

                        // 2. 将桌面设为 Owner Window（认贼作父）
                        // 这是让 Win+D 把你的窗口当作桌面附属品从而忽略最小化的核心
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
                        OLD_WNDPROC = Some(prev_wndproc);
                    }
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            // 清理工作
            WindowEvent::Destroyed => {
                #[cfg(target_os = "windows")]
                {
                    if let Ok(tauri_hwnd) = window.hwnd() {
                        unsafe {
                            let hwnd: HWND = std::mem::transmute(tauri_hwnd);
                            if let Some(old_proc) = OLD_WNDPROC {
                                SetWindowLongPtrW(hwnd, GWL_WNDPROC, old_proc);
                                OLD_WNDPROC = None;
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