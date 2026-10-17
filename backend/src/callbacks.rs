use core::fmt;

use crate::{
    events::{Button, EncoderEvent, KeyEvent, Slider},
    targetdevice::TargetDevice,
};

pub type ButtonCallback = Box<dyn FnMut(&mut TargetDevice, Button, bool) + Send>;

pub type KeyCallback = Box<dyn FnMut(&mut TargetDevice, KeyEvent) + Send>;

pub type SliderCallback = Box<dyn FnMut(&mut TargetDevice, Slider, i32) + Send>;

pub type EncoderCallback = Box<dyn FnMut(&mut TargetDevice, EncoderEvent) + Send>;

pub type MidiCallback = Box<dyn FnMut(&mut TargetDevice, &[u8]) + Send>;

#[derive(Default)]
pub struct Callbacks {
    pub button: Vec<ButtonCallback>,
    pub key: Vec<KeyCallback>,
    pub slider: Vec<SliderCallback>,
    pub encoder: Vec<EncoderCallback>,
    pub midi_merged: Vec<MidiCallback>,
}

impl Callbacks {
    pub fn append(&mut self, other: &mut Callbacks) {
        self.button.append(&mut other.button);
        self.key.append(&mut other.key);
        self.slider.append(&mut other.slider);
        self.encoder.append(&mut other.encoder);
        self.midi_merged.append(&mut other.midi_merged);
    }
}

impl fmt::Debug for Callbacks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Callbacks")
            .field("button", &self.button.len())
            .field("key", &self.key.len())
            .field("slider", &self.slider.len())
            .field("encoder", &self.encoder.len())
            .field("midi_merged", &self.midi_merged.len())
            .finish()
    }
}
