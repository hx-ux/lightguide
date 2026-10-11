use harmonia::PitchClass;
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
            display_name: HarmoniaUtils::normalize_pitch_names(&pitch),
            led_active,
        }
    }
}

pub struct HarmoniaUtils {}

impl HarmoniaUtils {
    pub const PITCH_CLASS_SELECTION: [PitchClass; 12] = [
        PitchClass::C,
        PitchClass::C_SHARP,
        PitchClass::D,
        PitchClass::D_SHARP,
        PitchClass::E,
        PitchClass::F,
        PitchClass::F_SHARP,
        PitchClass::G,
        PitchClass::G_SHARP,
        PitchClass::A,
        PitchClass::A_SHARP,
        PitchClass::B,
    ];

    pub fn normalize_pitch_names(pitch: &PitchClass) -> String {
        let name = match pitch.value() {
            0 => "C",
            1 => "C#",
            2 => "D",
            3 => "D#",
            4 => "E",
            5 => "F",
            6 => "F#",
            7 => "G",
            8 => "G#",
            9 => "A",
            10 => "A#",
            11 => "B",
            _ => unreachable!(),
        };
        name.to_string()
    }
}

// pub fn scale_alternate_name(name: String) -> ScaleKind {
//     let k = match name {
//         "Ionian (major)" => ScaleKind::Ionian,
//         "Aeolian (minor)" => ScaleKind::Aeolian,
//     };
// }
