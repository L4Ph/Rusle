use std::fmt;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY
};

/// Error type for key input operations
#[derive(Debug)]
pub struct KeyInputError {
    error_code: u32,
    message: String,
}

impl fmt::Display for KeyInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyInputError: {} (Code: {})", self.message, self.error_code)
    }
}

impl std::error::Error for KeyInputError {}

/// Sends virtual key press and release events for the specified key
///
/// # Arguments
///
/// * `new_key` - The Windows virtual key code to simulate
///
/// # Returns
///
/// * `Result<(), KeyInputError>` - Success or an error with details
pub fn send_key_input(new_key: u32) -> Result<(), KeyInputError> {
    let vk = VIRTUAL_KEY(new_key as u16);

    // Create input array with key down and key up events
    let inputs = [
        // Key down event
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0, // Not using scan code when using virtual key code
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        // Key up event
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0, // Not using scan code when using virtual key code
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];

    unsafe {
        let sent_count = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        
        // Check if all inputs were sent successfully
        if sent_count != inputs.len() as u32 {
            let error_code = GetLastError();
            return Err(KeyInputError {
                error_code: error_code.0,
                message: format!("SendInput failed. Sent {} of {}", sent_count, inputs.len()),
            });
        }
    }
    
    Ok(())
}