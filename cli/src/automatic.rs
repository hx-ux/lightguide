use std::collections::HashMap;

use backend::{
    display::TextAlignment,
    events::{Button, EncoderEvent, InputEvent, KeyEvent, Slider},
    keyboard_error::KeyboardError,
    keycolor::KeyColor,
    targetdevice::TargetDevice,
    utils::HarmoniaUtils,
};
use harmonia::{PitchClass, Scale, ScaleKind};

/// Number of endless knobs above the displays
pub const NUM_KNOBS: usize = 8;
/// Maximum knob value (MIDI CC range)
const KNOB_MAX: f32 = 127.0;
/// Raw encoder ticks per value step
const TICKS_PER_STEP: f32 = 4.0;
/// Raw encoder ticks per value step while Shift is held
const TICKS_PER_STEP_FINE: f32 = 32.0;
/// Display offset: knob n is shown on display n + DISPLAY_OFFSET
const DISPLAY_OFFSET: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Performance,
    Scale,
}

impl Mode {
    fn toggled(self) -> Self {
        match self {
            Mode::Performance => Mode::Scale,
            Mode::Scale => Mode::Performance,
        }
    }
}

pub struct AutomaticMode {
    mode: Mode,
    /// Knob values, 0.0 ..= 127.0 (kept fractional for smooth fine control)
    knobs: [f32; NUM_KNOBS],
    sustain_hold_notes: HashMap<u8, u8>,
    sustain: bool,
    sustained_keys: HashMap<u8, u8>,
    shift: bool,
    last_first_note: u8,
}

impl AutomaticMode {
    pub fn new(tg: &mut TargetDevice) -> Self {
        tg.root_note = PitchClass::new(0);
        tg.scale = ScaleKind::MajorPentatonic;
        tg.keys_hightlight_color = KeyColor::new(255, 0, 0);
        tg.keys_dim_color = KeyColor::new(0, 0, 0);
        tg.response_color = KeyColor::new(0, 100, 0);

        Self {
            mode: Mode::Performance,
            knobs: [0.0; NUM_KNOBS],
            sustain_hold_notes: HashMap::new(),
            sustain: false,
            sustained_keys: HashMap::new(),
            shift: false,
            last_first_note: 0,
        }
    }

    pub fn run(&mut self, tg: &mut TargetDevice) -> Result<(), KeyboardError> {
        if !tg.is_midi_connected() {
            eprintln!("warning: MIDI not connected - keys won't light up");
        }
        self.render_all_components(tg)?;

        loop {
            let events = tg.poll(5)?;
            for ev in events {
                self.handle(tg, ev)?;
            }
            if tg.first_note != self.last_first_note {
                self.render_leds(tg)?;
            }
        }
    }

    fn handle(&mut self, tg: &mut TargetDevice, ev: InputEvent) -> Result<(), KeyboardError> {
        match ev {
            InputEvent::Button { button, pressed } => self.on_button(tg, button, pressed),
            InputEvent::Key(key) => self.on_key(tg, key),
            InputEvent::Encoder(enc) => self.on_encoder(tg, enc),
            InputEvent::Slider {
                slider: Slider::Sustain,
                value,
            } => self.on_sustain(tg, value),
            InputEvent::Slider { .. } => Ok(()),
        }
    }

    /// stored and stay lit until the pedal goes back to not active
    fn on_sustain(&mut self, tg: &mut TargetDevice, value: i32) -> Result<(), KeyboardError> {
        let on = value >= 64;
        if on == self.sustain {
            return Ok(());
        }
        self.sustain = on;
        if on {
            self.sustained_keys = self.sustain_hold_notes.clone();
        } else {
            self.sustained_keys.clear();
        }
        self.render_leds(tg)
    }

    fn on_button(
        &mut self,
        tg: &mut TargetDevice,
        button: Button,
        pressed: bool,
    ) -> Result<(), KeyboardError> {
        if button == Button::Shift {
            self.shift = pressed;
            return Ok(());
        }
        if !pressed {
            return Ok(());
        }

        if button == Button::Scale {
            self.mode = self.mode.toggled();
            return self.render_all_components(tg);
        }

        if self.mode == Mode::Scale {
            match button {
                Button::NavigateLeft | Button::PageLeft => self.shift_root(tg, -1)?,
                Button::NavigateRight | Button::PageRight => self.shift_root(tg, 1)?,
                Button::NavigateUp => self.shift_scale(tg, -1)?,
                Button::NavigateDown => self.shift_scale(tg, 1)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn on_key(&mut self, tg: &mut TargetDevice, key: KeyEvent) -> Result<(), KeyboardError> {
        if key.pressed {
            self.sustain_hold_notes.insert(key.note, key.velocity);
            // While the sustain pedal is down every pressed key is kept lit
            if self.sustain {
                self.sustained_keys.insert(key.note, key.velocity);
            }
        } else {
            self.sustain_hold_notes.remove(&key.note);
        }
        self.render_leds(tg)
    }

    fn on_encoder(
        &mut self,
        tg: &mut TargetDevice,
        enc: EncoderEvent,
    ) -> Result<(), KeyboardError> {
        let knob = enc.index - 1;
        if knob >= NUM_KNOBS {
            return Ok(());
        }
        let ticks = if self.shift {
            TICKS_PER_STEP_FINE
        } else {
            TICKS_PER_STEP
        };
        let old = self.knob_value(knob);
        self.knobs[knob] = (self.knobs[knob] + enc.delta as f32 / ticks).clamp(0.0, KNOB_MAX);

        if self.mode == Mode::Performance && self.knob_value(knob) != old {
            self.draw_knob(tg, knob);
            tg.flush_display()?;
        }
        Ok(())
    }

    fn shift_root(&mut self, tg: &mut TargetDevice, delta: i32) -> Result<(), KeyboardError> {
        let root = (tg.root_note.value() as i32 + delta).rem_euclid(12) as u8;
        tg.root_note = PitchClass::new(root);
        self.render_all_components(tg)
    }

    fn shift_scale(&mut self, tg: &mut TargetDevice, delta: i32) -> Result<(), KeyboardError> {
        let all = ScaleKind::ALL;
        let current = all.iter().position(|k| *k == tg.scale).unwrap_or(0) as i32;
        let next = (current + delta).rem_euclid(all.len() as i32) as usize;
        tg.scale = all[next];
        self.render_all_components(tg)
    }

    pub fn knob_value(&self, knob: usize) -> u8 {
        self.knobs[knob].round() as u8
    }

    fn render_all_components(&mut self, tg: &mut TargetDevice) -> Result<(), KeyboardError> {
        self.render_display(tg)?;
        self.render_leds(tg)
    }

    fn render_leds(&mut self, tg: &mut TargetDevice) -> Result<(), KeyboardError> {
        self.last_first_note = tg.first_note;
        let scale = Scale::new(tg.root_note, tg.scale);
        let mut colors: Vec<KeyColor> = (0..tg.keys)
            .map(|pos| {
                let pc = tg.pitch_class_at(pos);
                if pc == tg.root_note {
                    tg.keys_hightlight_color
                } else if scale.contains(pc) {
                    tg.keys_hightlight_color
                } else {
                    tg.keys_dim_color
                }
            })
            .collect();

        for (&note, _) in self
            .sustained_keys
            .iter()
            .chain(self.sustain_hold_notes.iter())
        {
            let idx = note as i32 - tg.first_note as i32;
            if idx < 0 || idx as usize >= colors.len() {
                continue;
            }
            colors[idx as usize] = tg.response_color;
        }

        tg.set_key_leds(&colors)
    }
    fn render_display(&self, tg: &mut TargetDevice) -> Result<(), KeyboardError> {
        tg.display_buffer_mut().clear();
        match self.mode {
            Mode::Performance => {
                tg.display_text(0, 0, "PERFORM", TextAlignment::Center)?;
                tg.display_text(0, 1, "MODE", TextAlignment::Center)?;
                for knob in 0..NUM_KNOBS {
                    self.draw_knob(tg, knob);
                }
            }
            Mode::Scale => {
                let root = HarmoniaUtils::normalize_pitch_names(&tg.root_note);

                tg.display_text(0, 0, "SCALE", TextAlignment::Center)?;
                tg.display_text(0, 1, "MODE", TextAlignment::Center)?;
                tg.display_text(1, 0, "ROOT", TextAlignment::Center)?;
                tg.display_text(1, 1, &root, TextAlignment::Center)?;
                tg.display_text(2, 0, "SCALE", TextAlignment::Center)?;

                let mut index = 2;
                for p in tg.scale.name().split(" ") {
                    tg.display_text(index, 1, &p.to_uppercase(), TextAlignment::Center)?;
                    index = index + 1;
                }
            }
        }
        tg.flush_display()
    }

    /// Write bar + label + value of one knob into the display buffer.
    fn draw_knob(&self, tg: &mut TargetDevice, knob: usize) {
        let display = knob + DISPLAY_OFFSET;
        let value = self.knob_value(knob);
        let buf = tg.display_buffer_mut();
        buf.set_bar(display, value as f32 / KNOB_MAX);
        buf.set_text(
            display,
            0,
            &format!("KNOB {}", knob + 1),
            TextAlignment::Center,
        );
        buf.set_text(display, 1, &value.to_string(), TextAlignment::Center);
    }
}
