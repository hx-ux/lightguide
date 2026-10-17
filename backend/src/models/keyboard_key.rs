use harmonia::PitchClass;

use crate::extensions::pitch_extension::StringExt;

#[derive(PartialEq, Eq, Clone)]
pub enum KeyboardKeyType {
    White,
    Black,
}

#[derive(Clone)]

// Abstraction of an single key of the keyboard with its led
pub struct KeyboardKey {
    pub position: usize,
    pub ktype: KeyboardKeyType,
    pub display_name: String,
    pub led_active: bool,
}

impl KeyboardKey {
    pub fn new(
        position: usize,
        ktype: KeyboardKeyType,
        display_name: String,
        led_active: bool,
    ) -> Self {
        Self {
            position,
            ktype,
            display_name,
            led_active,
        }
    }

    // targets a key via its musical notation
    pub fn from_note(pitch: PitchClass, position: usize, led_active: bool) -> Self {
        Self {
            position,
            ktype: if (|pc: u8| matches!(pc, 1 | 3 | 6 | 8 | 10))(pitch.value()) {
                KeyboardKeyType::Black
            } else {
                KeyboardKeyType::White
            },
            display_name: pitch.display_text().to_string(),
            led_active,
        }
    }
}
