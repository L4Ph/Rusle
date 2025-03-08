use pc_keyboard::KeyCode;

/// Maps scancode to KeyCode for keyboard layout conversion
///
/// Returns Some(KeyCode) for mapped keys, None for unmapped keys
pub fn get_key_code_for_scancode(scancode: u8) -> Option<KeyCode> {
    match scancode {
        0x02 => Some(KeyCode::Key1),
        0x03 => Some(KeyCode::Key2),
        0x04 => Some(KeyCode::Key3),
        0x05 => Some(KeyCode::Key4),
        0x06 => Some(KeyCode::Key5),
        0x07 => Some(KeyCode::Key6),
        0x08 => Some(KeyCode::Key7),
        0x09 => Some(KeyCode::Key8),
        0x0A => Some(KeyCode::Key9),
        0x0B => Some(KeyCode::Key0),
        0x0C => Some(KeyCode::OemMinus),
        0x0D => Some(KeyCode::OemPlus),
        // Add more key mappings as needed
        _ => None, // Return None for unmapped keys
    }
}