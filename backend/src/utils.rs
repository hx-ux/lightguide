use harmonia::{PitchClass, ScaleKind};

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
    // pub fn scale_alternate_name(name: str) -> ScaleKind {
    //     let k = match name {
    //         "Ionian (major)" => ScaleKind::Ionian,
    //         "Aeolian (minor)" => ScaleKind::Aeolian,
    //     };
    // }
}
