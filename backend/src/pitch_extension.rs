use harmonia::PitchClass;

use serde::{Deserialize, Deserializer, Serializer};

pub trait StringExt {
    fn display_text(&self) -> &str;
}

impl StringExt for PitchClass {
    fn display_text(&self) -> &str {
        let name = match self.value() {
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

        name
    }
}

pub mod pitch_serde {
    use super::*;

    pub fn serialize<S>(pitch: &PitchClass, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let name = pitch.display_text();
        serializer.serialize_str(name)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PitchClass, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let value = match s.as_str() {
            "C" => 0,
            "C#" | "Db" => 1,
            "D" => 2,
            "D#" | "Eb" => 3,
            "E" => 4,
            "F" => 5,
            "F#" | "Gb" => 6,
            "G" => 7,
            "G#" | "Ab" => 8,
            "A" => 9,
            "A#" | "Bb" => 10,
            "B" => 11,
            _ => return Err(serde::de::Error::custom(format!("unknown note name: {s}"))),
        };
        Ok(PitchClass::from(value))
    }
}

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
