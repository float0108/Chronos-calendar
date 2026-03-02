use tauri::Manager;
// 1. 引入正确的状态枚举 EffectState
use tauri::window::{Effect, EffectState};
// 2. 引入正确的配置结构体 WindowEffectsConfig
use tauri::utils::config::WindowEffectsConfig;

#[tauri::command]
fn set_window_vibrancy(window: tauri::WebviewWindow, enable: bool) -> Result<(), String> {
    if enable {
        // Windows 平台使用 Mica 或 Acrylic 效果
        #[cfg(target_os = "windows")]
        {
            let effects = WindowEffectsConfig {
                effects: vec![Effect::Mica],
                state: Some(EffectState::Active),
                ..Default::default()
            };
            window.set_effects(Some(effects)).map_err(|e| e.to_string())?;
        }
        
        // macOS 使用 Vibrancy 效果
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
        // 禁用效果（显式标注类型以防 Rust 推断失败）
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}