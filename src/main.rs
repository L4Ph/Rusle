use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WM_KEYDOWN,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY
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
            if kbdllhookstruct.vkCode == 'A' as u32 {
                // A キーが押されたら B キー入力を送信
                let b_input = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY('B' as u16), // B の仮想キーコード
                            wScan: 0,
                            dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0), // 押下イベント
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };

                // B キーリリースイベントも送信
                let b_input_up = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY('B' as u16),
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP, // キーアップイベント
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };

                SendInput(&[b_input, b_input_up], std::mem::size_of::<INPUT>() as i32);
                return LRESULT(1); // A の入力を遮断
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
