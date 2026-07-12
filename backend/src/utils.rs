use harmonia::PitchClass;
#[derive(PartialEq, Eq, Clone)]
pub enum KeyboardKeysType {
    White,
    Black,
}

#[derive(Clone)]

pub struct KeyboardKey {
    pub position: usize,
    pub ktype: KeyboardKeysType,
    pub display_name: String,
    pub active: bool,
}

impl KeyboardKey {
    pub fn new(
        position: usize,
        ktype: KeyboardKeysType,
        display_name: String,
        active: bool,
    ) -> Self {
        Self {
            position,
            ktype,
            display_name,
            active,
        }
    }
}

pub struct HarmoniaUtils {}

impl HarmoniaUtils {
    pub fn pitch_class_selection() -> Vec<PitchClass> {
        let mut g: Vec<PitchClass> = vec![];
        g.push(PitchClass::C);
        g.push(PitchClass::C_SHARP);
        g.push(PitchClass::D);
        g.push(PitchClass::D_SHARP);
        g.push(PitchClass::E);
        g.push(PitchClass::F);
        g.push(PitchClass::F_SHARP);
        g.push(PitchClass::G);
        g.push(PitchClass::G_SHARP);
        g.push(PitchClass::A);
        g.push(PitchClass::A_SHARP);
        g.push(PitchClass::B);
        g
    }

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
