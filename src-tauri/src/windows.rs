// === Windows 平台特定代码 ===

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    GetShellWindow, GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos,
    CallWindowProcW, GWLP_HWNDPARENT, GWL_EXSTYLE, GWL_WNDPROC, HWND_BOTTOM,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    WM_WINDOWPOSCHANGING, WINDOWPOS, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::sync::{Mutex, LazyLock};

/// 存储各窗口的原始窗口过程
#[cfg(target_os = "windows")]
static OLD_WNDPROCS: LazyLock<Mutex<HashMap<isize, isize>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// 窗口子类化过程：拦截系统改变层级，强行锁定在底层
#[cfg(target_os = "windows")]
unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_WINDOWPOSCHANGING {
        let wp = &mut *(lparam.0 as *mut WINDOWPOS);
        if (wp.flags.0 & SWP_NOZORDER.0) == 0 {
            wp.hwndInsertAfter = HWND_BOTTOM;
        }
    }

    let old_proc = OLD_WNDPROCS.lock().unwrap().get(&(hwnd.0 as isize)).copied();
    if let Some(old_proc) = old_proc {
        CallWindowProcW(std::mem::transmute(old_proc), hwnd, msg, wparam, lparam)
    } else {
        LRESULT(0)
    }
}

/// 应用窗口设置（隐藏任务栏、置底、子类化钩子）
#[cfg(target_os = "windows")]
pub unsafe fn apply_window_settings(hwnd: HWND) {
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

/// 恢复窗口原始过程
#[cfg(target_os = "windows")]
pub unsafe fn restore_window_proc(hwnd: HWND) {
    if let Some(old_proc) = OLD_WNDPROCS.lock().unwrap().remove(&(hwnd.0 as isize)) {
        SetWindowLongPtrW(hwnd, GWL_WNDPROC, old_proc);
    }
}

/// 非 Windows 平台的空实现
#[cfg(not(target_os = "windows"))]
pub unsafe fn apply_window_settings(_hwnd: ()) {}

#[cfg(not(target_os = "windows"))]
pub unsafe fn restore_window_proc(_hwnd: ()) {}
