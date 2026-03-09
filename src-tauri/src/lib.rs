use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use font_kit::source::SystemSource;
use std::collections::HashSet;
use tauri_plugin_autostart::MacosLauncher;

// === Windows 窗口样式和 Hook 相关的 API 引入 ===
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, SetWindowLongPtrW, GetWindowLongPtrW, GetShellWindow,
    HWND_BOTTOM, GWL_EXSTYLE, GWLP_HWNDPARENT, GWL_WNDPROC,
    SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_APPWINDOW,
    CallWindowProcW, WM_WINDOWPOSCHANGING, WINDOWPOS, SWP_NOZORDER,
};

// === 全局变量：存储原始窗口过程 ===
#[cfg(target_os = "windows")]
static mut OLD_WNDPROC: Option<isize> = None;

// === 窗口子类化消息处理函数 ===
// 拦截 WM_WINDOWPOSCHANGING，在系统改变窗口层级前强行锁定到底层
#[cfg(target_os = "windows")]
unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_WINDOWPOSCHANGING {
        let wp = &mut *(lparam.0 as *mut WINDOWPOS);

        // 检查系统是否试图改变 Z-Order（如果 SWP_NOZORDER 标志不存在，说明要改层级）
        if (wp.flags.0 & SWP_NOZORDER.0) == 0 {
            // 强行篡改目的地：无论系统想把我放到哪，都给我回到底层！
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
            let family_name = font.family_name();
            font_names.insert(family_name);
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
    // 如果窗口已存在，聚焦它
    if let Some(window) = app.get_webview_window("settings") {
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    // 创建设置窗口
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

    // 隐藏任务栏图标
    let _ = settings_window.set_skip_taskbar(true);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 当尝试启动第二个实例时，聚焦主窗口
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

            // 隐藏任务栏图标
            let _ = window.set_skip_taskbar(true);

            // Windows 平台设置
            #[cfg(target_os = "windows")]
            {
                if let Ok(tauri_hwnd) = window.hwnd() {
                    unsafe {
                        let hwnd: HWND = std::mem::transmute(tauri_hwnd);

                        // 1. 设置 WS_EX_TOOLWINDOW 样式（隐藏任务栏图标）
                        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                        let new_ex_style = (ex_style | WS_EX_TOOLWINDOW.0 as isize)
                            & !(WS_EX_APPWINDOW.0 as isize);
                        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex_style);

                        // 2. 核心黑科技：将桌面设为 Owner Window
                        // 这不是 SetParent！保留独立窗口的所有交互能力，
                        // 但强制在层级上绑定桌面，Win+D 时自动跟随桌面显示
                        let shell_hwnd = GetShellWindow();
                        if !shell_hwnd.0.is_null() {
                            SetWindowLongPtrW(hwnd, GWLP_HWNDPARENT, shell_hwnd.0 as isize);
                            eprintln!("[Setup] 已将桌面设置为 Owner Window");
                        }

                        // 3. 初始置底
                        let _ = SetWindowPos(
                            hwnd,
                            HWND_BOTTOM,
                            0, 0, 0, 0,
                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                        );

                        // 4. 挂载 Z-Order 锁定钩子（拦截 WM_WINDOWPOSCHANGING）
                        let prev_wndproc = SetWindowLongPtrW(
                            hwnd,
                            GWL_WNDPROC,
                            subclass_proc as *const () as isize,
                        );
                        OLD_WNDPROC = Some(prev_wndproc);
                        eprintln!("[Setup] 已挂载 Z-Order 锁定钩子");
                    }
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            // 窗口关闭时恢复原始窗口过程
            WindowEvent::CloseRequested { .. } => {
                #[cfg(target_os = "windows")]
                {
                    if let Ok(tauri_hwnd) = window.hwnd() {
                        unsafe {
                            let hwnd: HWND = std::mem::transmute(tauri_hwnd);
                            // 恢复原始窗口过程
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
