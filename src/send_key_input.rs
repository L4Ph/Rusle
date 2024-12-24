use windows::Win32::Foundation::GetLastError;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY
};

/// new_key に対応する仮想キーの押下と解放イベントを送信する関数
pub fn send_key_input(new_key: u32) {
    let vk = VIRTUAL_KEY(new_key as u16);

    let inputs = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0, // 仮想キーコードを使用するため、スキャンコードは設定しない
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0, // 仮想キーコードを使用するため、スキャンコードは設定しない
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];

    unsafe {
        let sent_count = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if sent_count != inputs.len() as u32 {
            let error_code = GetLastError();
            eprintln!("SendInput failed. Sent {} of {}, Error Code: {:?}", sent_count, inputs.len(), error_code);
            // 必要に応じて、ユーザーにエラーメッセージを表示したり、リトライ処理を行ったりする
        }
    }
}