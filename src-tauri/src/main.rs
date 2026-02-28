// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // 禁用 WebView2 硬件加速，解决透明窗口闪烁问题
    // std::env::set_var("WEBVIEW2_DISABLE_GPU", "1");
    std::env::set_var("WEBVIEW2_DEFAULT_BACKGROUND_COLOR", "0");
    
    chronos_lib::run();
}
