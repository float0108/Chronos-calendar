use tauri::{Manager, WindowEvent};
// 1. 引入正确的状态枚举 EffectState
use tauri::window::{Effect, EffectState};
// 2. 引入正确的配置结构体 WindowEffectsConfig
use tauri::utils::config::WindowEffectsConfig;

// === Windows 置底相关的 API 引入 ===
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
};

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

#[tauri::command]
fn set_window_vibrancy(window: tauri::WebviewWindow, enable: bool) -> Result<(), String> {
    if enable {
        #[cfg(target_os = "windows")]
        {
            let effects = WindowEffectsConfig {
                effects: vec![Effect::Mica],
                state: Some(EffectState::Active),
                ..Default::default()
            };
            window.set_effects(Some(effects)).map_err(|e| e.to_string())?;
        }
        
        #[cfg(target_os = "macos")]
        {
            let effects = WindowEffectsConfig {
                effects: vec![Effect::Popover],
                state: Some(EffectState::Active),
                ..Default::default()
            };
            window.set_effects(Some(effects)).map_err(|e| e.to_string())?;
        }
    } else {
        window.set_effects(None::<WindowEffectsConfig>).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![set_window_vibrancy])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            
            // 隐藏任务栏图标
            let _ = window.set_skip_taskbar(true);
            
            // 默认启用毛玻璃效果
            #[cfg(target_os = "windows")]
            {
                let effects = WindowEffectsConfig {
                    effects: vec![Effect::Mica],
                    state: Some(EffectState::Active),
                    ..Default::default()
                };
                let _ = window.set_effects(Some(effects));
                
                // 使用宏：由于 window 包含 .hwnd() 方法，完美展开！
                push_to_bottom!(window);
            }
            
            #[cfg(target_os = "macos")]
            {
                let effects = WindowEffectsConfig {
                    effects: vec![Effect::Popover],
                    state: Some(EffectState::Active),
                    ..Default::default()
                };
                let _ = window.set_effects(Some(effects));
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
                if window.is_minimized().unwrap_or(false) {
                    // 立刻恢复窗口（反最小化）
                    let _ = window.unminimize();
                    // 恢复后立即再次压回底层
                    push_to_bottom!(window);
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}