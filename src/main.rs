mod send_key_input;
mod key_mapping;

use key_mapping::get_key_code_for_scancode;
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
    HandleControl, KeyEvent, Keyboard, ScancodeSet1,
};
use lazy_static::lazy_static;
use std::sync::Mutex;

/// Flag to control the application's running state
static RUNNING: AtomicBool = AtomicBool::new(true);

// Create static instances of the keyboards to avoid recreating them on every key press
lazy_static! {
    static ref JIS_KEYBOARD: Mutex<Keyboard<Jis109Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(ScancodeSet1::new(), Jis109Key, HandleControl::Ignore)
    );
    
    static ref US_KEYBOARD: Mutex<Keyboard<Us104Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(ScancodeSet1::new(), Us104Key, HandleControl::Ignore)
    );
}

/// Windows keyboard hook procedure that intercepts keystrokes
///
/// This function is called by Windows when a keyboard event occurs.
/// It converts JIS keyboard layout keys to US layout and prevents the original key from being processed.
extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    unsafe {
        // Only process keydown events and when hook code is non-negative
        if n_code >= 0 && w_param.0 == WM_KEYDOWN as usize {
            let kbdllhookstruct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
            let scancode = kbdllhookstruct.scanCode as u8;

            // Get the KeyCode for this scancode, or pass through if not mapped
            let Some(key_code) = get_key_code_for_scancode(scancode) else {
                return CallNextHookEx(HHOOK(null_mut()), n_code, w_param, l_param);
            };

            // Create the keyboard event with the mapped key
            let key_event = KeyEvent {
                code: key_code,
                state: pc_keyboard::KeyState::Down,
            };

            // Process the key through both JIS and US keyboard layouts using the static instances
            let process_through_keyboards = || -> bool {
                // Try to lock the keyboards and process the key event
                let mut jis_keyboard = JIS_KEYBOARD.lock().ok()?;
                let mut us_keyboard = US_KEYBOARD.lock().ok()?;
                
                let _jis_key = jis_keyboard.process_keyevent(key_event.clone())?;
                let _us_key = us_keyboard.process_keyevent(key_event)?;
                
                // If we reach here, both keyboards have processed the key successfully
                Some(true)
            };

            // If key mapping is successful, send the key input
            if process_through_keyboards() == Some(true) {
                if let Err(e) = send_key_input(kbdllhookstruct.vkCode) {
                    eprintln!("Error sending key input: {}", e);
                }
                return LRESULT(1); // Block the original key
            }
        }
        
        // Pass the event to the next hook
        CallNextHookEx(HHOOK(null_mut()), n_code, w_param, l_param)
    }
}

/// Setup function to register keyboard hook
fn setup_keyboard_hook() -> Result<HHOOK, String> {
    unsafe {
        match SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            HINSTANCE(null_mut()),
            0,
        ) {
            Ok(hook) => Ok(hook),
            Err(e) => Err(format!("Failed to set keyboard hook: {:?}", e)),
        }
    }
}

/// Main event loop to process Windows messages
fn run_message_loop() {
    unsafe {
        while RUNNING.load(Ordering::Relaxed) {
            let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
            if GetMessageW(&mut msg, HWND(null_mut()), 0, 0).into() {
                let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
            }
        }
    }
}

/// Clean up hook on application exit
fn cleanup_hook(hook: HHOOK) -> Result<(), String> {
    unsafe {
        match UnhookWindowsHookEx(hook) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to unhook Windows hook: {:?}", e)),
        }
    }
}

/// Register Ctrl+C handler to enable graceful shutdown
fn setup_ctrl_c_handler() {
    ctrlc::set_handler(|| {
        println!("Received Ctrl+C, shutting down...");
        RUNNING.store(false, Ordering::Relaxed);
        
        // Post a dummy message to unblock GetMessageW
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::PostMessageA(
                HWND(null_mut()),
                windows::Win32::UI::WindowsAndMessaging::WM_NULL,
                WPARAM(0),
                LPARAM(0),
            ).ok();
        }
    }).expect("Error setting Ctrl+C handler");
}

fn main() {
    println!("Starting keyboard layout converter...");
    
    // Set up Ctrl+C handler for graceful shutdown
    setup_ctrl_c_handler();
    
    // Setup keyboard hook
    let hook = match setup_keyboard_hook() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    println!("Hook installed. Press Ctrl+C to exit.");

    // Run message loop
    run_message_loop();

    // Clean up
    if let Err(e) = cleanup_hook(hook) {
        eprintln!("Error during cleanup: {}", e);
    }
    
    println!("Application terminated successfully.");
}
