mod send_key_input;
use send_key_input::send_key_input;

use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WM_KEYDOWN,
};

use pc_keyboard::{
    layouts::{Jis109Key, Us104Key},
    HandleControl, Keyboard, ScancodeSet1, KeyEvent, KeyCode, DecodedKey,
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
            let scancode = kbdllhookstruct.scanCode as u8;

            let mut jis_keyboard = Keyboard::new(ScancodeSet1::new(), Jis109Key, HandleControl::Ignore);
            let mut us_keyboard = Keyboard::new(ScancodeSet1::new(), Us104Key, HandleControl::Ignore);

            let key_event = KeyEvent {
                code: match scancode {
                    0x0B => KeyCode::Key0, // 0
                    0x02 => KeyCode::Key1, // 1
                    _ => return CallNextHookEx(HHOOK(null_mut()), n_code, w_param, l_param), // 未対応キーはスルー
                },
                state: pc_keyboard::KeyState::Down,
            };

            if let Some(_jis_key) = jis_keyboard.process_keyevent(key_event.clone()) {
                if let Some(us_key) = us_keyboard.process_keyevent(key_event) {
                    match us_key {
                        DecodedKey::Unicode(c) => send_key_input(kbdllhookstruct.vkCode, c as u32),
                        DecodedKey::RawKey(key_code) => send_key_input(kbdllhookstruct.vkCode, key_code as u32),
                    }
                    return LRESULT(1); // 入力を遮断
                }
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
