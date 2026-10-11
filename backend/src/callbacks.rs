use core::fmt;

use crate::{
    events::{Button, EncoderEvent, KeyEvent, Slider},
    targetdevice::TargetDevice,
};

/// Callback for button presses/releases: `(device, button, pressed)`
pub type ButtonCallback = Box<dyn FnMut(&mut TargetDevice, Button, bool) + Send>;

pub type KeyCallback = Box<dyn FnMut(&mut TargetDevice, KeyEvent) + Send>;

pub type SliderCallback = Box<dyn FnMut(&mut TargetDevice, Slider, i32) + Send>;

pub type EncoderCallback = Box<dyn FnMut(&mut TargetDevice, EncoderEvent) + Send>;

#[derive(Default)]
pub struct Callbacks {
    pub button: Vec<ButtonCallback>,
    pub key: Vec<KeyCallback>,
    pub slider: Vec<SliderCallback>,
    pub encoder: Vec<EncoderCallback>,
}

impl Callbacks {
    fn append(&mut self, other: &mut Callbacks) {
        self.button.append(&mut other.button);
        self.key.append(&mut other.key);
        self.slider.append(&mut other.slider);
        self.encoder.append(&mut other.encoder);
    }
}

impl fmt::Debug for Callbacks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Callbacks")
            .field("button", &self.button.len())
            .field("key", &self.key.len())
            .field("slider", &self.slider.len())
            .field("encoder", &self.encoder.len())
            .finish()
    }
}
