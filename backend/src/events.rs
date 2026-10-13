#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    /// Pushing the main (browse) encoder
    MainEncoderPress,
    /// Touching the main (browse) encoder
    TouchEncoder(u8),
    /// Touching the main encoder
    TouchEncoderMain,

    PresetUp,
    Enter,
    PresetDown,
    Browse,
    Instance,
    OctaveDown,
    OctaveUp,

    Stop,
    Rec,
    Play,
    NavigateRight,
    NavigateDown,
    NavigateLeft,
    Back,
    NavigateUp,

    Shift,
    Scale,
    Arp,
    Loop,
    PageRight,
    PageLeft,
    Rwd,
    Ffw,

    /// Bit that is set in the report but not mapped (yet)
    Unknown(u8),
}

impl Button {
    /// Map a bit index of the HID button bitfield to a [`Button`].
    pub fn from_bit(bit: u8) -> Self {
        use Button::*;
        match bit {
            0 => MainEncoderPress,
            1 => PresetUp,
            2 => Enter,
            3 => PresetDown,
            4 => Browse,
            5 => Instance,
            6 => OctaveDown,
            7 => OctaveUp,
            8 => Stop,
            9 => Rec,
            10 => Play,
            11 => NavigateRight,
            12 => NavigateDown,
            13 => NavigateLeft,
            14 => Back,
            15 => NavigateUp,
            16 => Shift,
            17 => Scale,
            18 => Arp,
            19 => Loop,
            20 => PageRight,
            21 => PageLeft,
            22 => Rwd,
            23 => Ffw,
            24..=31 => TouchEncoder(bit - 24),
            32 => TouchEncoderMain,
            other => Unknown(other),
        }
    }
}

/// A piano key was pressed or released.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    /// MIDI note number as sent by the keyboard (already includes octave shift)
    pub note: u8,
    /// Physical key index (0 = left-most key), `None` if it can't be mapped
    pub key_index: Option<usize>,
    /// Velocity 1..=127 on press, release velocity (often 0) on release
    pub velocity: u8,
    /// `true` = key down, `false` = key up
    pub pressed: bool,
    /// MIDI channel 0..=15
    pub channel: u8,
}

/// Continuous controllers (touch strips, pedals, ...), received via MIDI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slider {
    /// Pitch touch strip. Value range: -8192..=8191 (0 = centre)
    PitchBend,
    /// Modulation touch strip (CC 1). Value range 0..=127
    Modulation,
    /// Sustain pedal (CC 64). Value range 0..=127
    Sustain,
    /// Expression pedal (CC 11). Value range 0..=127
    Expression,
    /// Channel aftertouch. Value range 0..=127
    Aftertouch,
    /// Any other control change. Value range 0..=127
    ControlChange(u8),
}

/// One of the endless knobs was turned (HID).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncoderEvent {
    /// 0 = main (browse) encoder, 1..=8 = the knobs above the displays
    pub index: usize,
    /// Signed number of steps since the last report (positive = clockwise)
    pub delta: i32,
    /// Raw absolute value reported by the hardware
    pub raw: u16,
}

/// Every event the device can emit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    Button { button: Button, pressed: bool },
    Key(KeyEvent),
    Slider { slider: Slider, value: i32 },
    Encoder(EncoderEvent),
}
