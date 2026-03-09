use tauri::{Manager, WindowEvent, WebviewUrl, WebviewWindowBuilder};
use font_kit::source::SystemSource;
use std::collections::HashSet;
use tauri_plugin_autostart::MacosLauncher;

// === Windows 置底相关的 API 引入 ===
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    RegisterShellHookWindow, RegisterWindowMessageW, SetWindowLongPtrW,
    CallWindowProcW, GWL_WNDPROC,
    ShowWindow, SW_SHOWNOACTIVATE,
    HSHELL_WINDOWACTIVATED,
};

// HSHELL_RUDEAPPACTIVATED 在 windows crate 中未暴露，手动定义
// 这个值表示"粗鲁激活" - Win+D 显示桌面时触发
#[cfg(target_os = "windows")]
const HSHELL_RUDEAPPACTIVATED: i32 = 0x8004; // 32772

// === Shell Hook 全局状态 ===
#[cfg(target_os = "windows")]
static mut OLD_WNDPROC: Option<isize> = None;
#[cfg(target_os = "windows")]
static mut SHELL_HOOK_MSG: u32 = 0;

// === 使用宏代替泛型函数，完美解决 Window 和 WebviewWindow 的类型差异 ===
macro_rules! push_to_bottom {
    ($window:expr) => {
        #[cfg(target_os = "windows")]
        {
            if let Ok(tauri_hwnd) = $window.hwnd() {
                unsafe {
                    // 强制内存转换，忽略版本冲突
                    let hwnd: HWND = std::mem::transmute(tauri_hwnd);
                    let _ = SetWindowPos(
                        hwnd,
                        HWND_BOTTOM,
                        0, 0, 0, 0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                    );
                }
            }
        }
    };
}

// === Shell Hook 子类化消息处理函数 ===
#[cfg(target_os = "windows")]
unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // 检查是否为 Shell Hook 消息
    if msg == SHELL_HOOK_MSG && SHELL_HOOK_MSG != 0 {
        let code = wparam.0 as i32;

        // 打印所有 Shell Hook 消息用于调试
        eprintln!("[ShellHook] 收到消息: code=0x{:X} ({})", code, code);

        // HSHELL_WINDOWACTIVATED: 窗口激活事件
        if code == HSHELL_WINDOWACTIVATED as i32 {
            eprintln!("[ShellHook] HSHELL_WINDOWACTIVATED 触发");
        }

        // HSHELL_RUDEAPPACTIVATED: Win+D 触发时的关键信号
        if code == HSHELL_RUDEAPPACTIVATED {
            eprintln!("[ShellHook] HSHELL_RUDEAPPACTIVATED 触发 - Win+D 检测到!");
        }

        // 对两种情况都尝试恢复窗口
        if code == HSHELL_WINDOWACTIVATED as i32 || code == HSHELL_RUDEAPPACTIVATED {
            // 使用 SW_SHOWNOACTIVATE 显示窗口但不抢夺焦点
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

            // 恢复后再次压到底层
            let _ = SetWindowPos(
                hwnd,
                HWND_BOTTOM,
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            eprintln!("[ShellHook] 已执行窗口恢复和置底操作");
        }
    }

    // 将消息传递给原有处理程序
    if let Some(old_proc) = OLD_WNDPROC {
        CallWindowProcW(
            std::mem::transmute(old_proc),
            hwnd,
            msg,
            wparam,
            lparam,
        )
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

            // Windows 置底 + Shell Hook 注册
            #[cfg(target_os = "windows")]
            {
                push_to_bottom!(window);

                // 注册 Shell Hook 和窗口子类化
                if let Ok(tauri_hwnd) = window.hwnd() {
                    unsafe {
                        let hwnd: HWND = std::mem::transmute(tauri_hwnd);

                        // 1. 注册 Shell Hook 窗口
                        let result = RegisterShellHookWindow(hwnd);
                        eprintln!("[Setup] RegisterShellHookWindow 结果: {:?}", result);

                        // 2. 获取 Shell Hook 消息 ID
                        SHELL_HOOK_MSG = RegisterWindowMessageW(windows::core::w!("SHELLHOOK"));
                        eprintln!("[Setup] SHELL_HOOK_MSG ID: {}", SHELL_HOOK_MSG);

                        // 3. 子类化窗口以拦截消息
                        let prev_wndproc = SetWindowLongPtrW(
                            hwnd,
                            GWL_WNDPROC,
                            subclass_proc as *const () as isize,
                        );
                        OLD_WNDPROC = Some(prev_wndproc);
                        eprintln!("[Setup] 原始 WndProc: {:?}", prev_wndproc);
                    }
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            // 1. 解决点击时的置底问题
            WindowEvent::Focused(true) => {
                // 使用宏：同样无缝支持事件里传来的原生 Window
                push_to_bottom!(window);
            }

            // 2. 核心修复：解决 Win + D 导致窗口丢失的问题
            WindowEvent::Resized(_) => {
                // 检查窗口是否处于最小化状态（Win + D 的典型结果）
                let is_min = window.is_minimized().unwrap_or(false);
                eprintln!("[Resized] is_minimized: {}", is_min);
                if is_min {
                    eprintln!("[Resized] 检测到最小化，正在恢复窗口...");
                    // 立刻恢复窗口（反最小化）
                    let _ = window.unminimize();
                    // 恢复后立即再次压回底层
                    push_to_bottom!(window);
                    eprintln!("[Resized] 窗口已恢复并置底");
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}