mod key_input;
use key_input::send_key_input;

use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WM_KEYDOWN,
};

static RUNNING: AtomicBool = AtomicBool::new(true);

extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    unsafe {
        if n_code >= 0 && w_param.0 == WM_KEYDOWN as usize {
            let kbdllhookstruct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            if kbdllhookstruct.vkCode == 'D' as u32 {
                send_key_input('D', 'F'); // 'D' を 'F' に変換
                return LRESULT(1); // 'D' の入力を遮断
            }
        }
        CallNextHookEx(HHOOK(null_mut()), n_code, w_param, l_param)
    }
}

fn main() {
    unsafe {
        let hook: HHOOK = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            HINSTANCE(null_mut()),
            0,
        )
        .expect("Failed to set hook");

        println!("Hook installed. Press Ctrl+C to exit.");

        while RUNNING.load(Ordering::Relaxed) {
            let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
            if GetMessageW(&mut msg, HWND(null_mut()), 0, 0).into() {
                let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
            }
        }

        if !UnhookWindowsHookEx(hook).is_ok() {
            panic!("Failed to uninstall hook");
        }
    }
}