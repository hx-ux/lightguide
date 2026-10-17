use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use backend::{
    display::TextAlignment,
    events::{Button, EncoderEvent, InputEvent, KeyEvent, Slider},
    extensions::pitch_extension::StringExt,
    keyboard_error::KeyboardError,
    models::keycolor::KeyColor,
    targetdevice::TargetDevice,
};
use harmonia::{PitchClass, Scale, ScaleKind};

use crate::modes::automatic::arp::{ArpPattern, ArpSettings};

// use crate::modes::automatic::;

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
    Arp,
}

impl Mode {
    pub fn toggle_Scale_Mode(self) -> Self {
        if self == Mode::Performance {
            return Mode::Scale;
        }
        return Mode::Performance;
    }

    pub fn toggle_arp_mode(self) -> Self {
        if self == Mode::Performance {
            return Mode::Arp;
        }
        return Mode::Performance;
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
    arp_settings: ArpSettings,
    arp_active_note: Option<(u8, Instant, Duration)>, // (note, start_time, gate_duration)
}

impl AutomaticMode {
    pub fn new() -> Self {
        Self {
            mode: Mode::Performance,
            knobs: [0.0; NUM_KNOBS],
            sustain_hold_notes: HashMap::new(),
            sustain: false,
            sustained_keys: HashMap::new(),
            shift: false,
            last_first_note: 0,
            arp_settings: ArpSettings::default(), // arp_settings:
            arp_active_note: None,
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
            self.tick_arpeggiator(tg)?;
            if tg.first_note != self.last_first_note {
                self.render_leds(tg)?;
            }
        }
    }
    /// Ticks the arpeggiator clock, triggers note-on/note-off, and updates key LEDs
    fn tick_arpeggiator(&mut self, tg: &mut TargetDevice) -> Result<(), KeyboardError> {
        if !self.arp_settings.active {
            return Ok(());
        }

        let now = Instant::now();

        // Check if an active note needs note-off due to gate expiration
        if let Some((_note, start_time, gate_duration)) = self.arp_active_note {
            if now.duration_since(start_time) >= gate_duration {
                self.stop_arp_note(tg)?;
                self.render_leds(tg)?;
            }
        }

        // Active notes pool from currently held keys (or sustained keys)
        let active_pool: Vec<u8> = if self.sustain {
            self.sustained_keys.keys().copied().collect()
        } else {
            self.arp_settings.held_keys.iter().copied().collect()
        };

        if active_pool.is_empty() {
            if self.arp_active_note.is_some() {
                self.stop_arp_note(tg)?;
                self.render_leds(tg)?;
            }
            return Ok(());
        }

        let step_dur = self.arp_settings.step_duration();

        // Check if time has arrived for the next arpeggiator step
        if now.duration_since(self.arp_settings.last_tick) >= step_dur {
            self.arp_settings.last_tick = now;

            // Stop previous note if still ringing
            self.stop_arp_note(tg)?;

            // Build full note list expanded across octaves
            let mut base_notes = active_pool;
            base_notes.sort_unstable();

            let mut notes = Vec::new();
            for oct in 0..self.arp_settings.octaves {
                for &n in &base_notes {
                    let trans = n + (oct as u8 * 12);
                    if trans <= 127 {
                        notes.push(trans);
                    }
                }
            }

            if notes.is_empty() {
                return Ok(());
            }

            let pattern = self.arp_settings.pattern;
            let note = match pattern {
                ArpPattern::Up => {
                    let idx = self.arp_settings.arp_step_index % notes.len();
                    self.arp_settings.arp_step_index =
                        (self.arp_settings.arp_step_index + 1) % notes.len();
                    notes[idx]
                }
                ArpPattern::Down => {
                    let idx = (notes.len() - 1 - (self.arp_settings.arp_step_index % notes.len()))
                        % notes.len();
                    self.arp_settings.arp_step_index =
                        (self.arp_settings.arp_step_index + 1) % notes.len();
                    notes[idx]
                }
                ArpPattern::UpDown => {
                    if notes.len() == 1 {
                        notes[0]
                    } else {
                        let total_steps = (notes.len() - 1) * 2;
                        let phase = self.arp_settings.arp_step_index % total_steps;
                        self.arp_settings.arp_step_index =
                            (self.arp_settings.arp_step_index + 1) % total_steps;
                        if phase < notes.len() {
                            notes[phase]
                        } else {
                            notes[total_steps - phase]
                        }
                    }
                }
            };

            // Trigger Note On
            if let Some(out) = tg.midi_out_mut() {
                let _ = out.note_on(0, note, 100);
            }

            let gate_dur =
                Duration::from_secs_f32(step_dur.as_secs_f32() * self.arp_settings.gate as f32);
            self.arp_active_note = Some((note, now, gate_dur));

            // Visual feedback: Update LEDs so the playing arpeggiator note lights up!
            self.render_leds(tg)?;
        }

        Ok(())
    }

    fn stop_arp_note(&mut self, tg: &mut TargetDevice) -> Result<(), KeyboardError> {
        if let Some((note, _, _)) = self.arp_active_note.take() {
            if let Some(out) = tg.midi_out_mut() {
                let _ = out.note_off(0, note, 64);
            }
        }
        Ok(())
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
            InputEvent::Slider { slider, value } => {
                // Emulate the slider midi values
                tg.send_midi_event(&InputEvent::Slider { slider, value });
                Ok(())
            }
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
            self.mode = self.mode.toggle_Scale_Mode();
            return self.render_all_components(tg);
        }

        if button == Button::Arp {
            self.mode = self.mode.toggle_arp_mode();
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
        // 3. Active arpeggiator step visual feedback: bright highlight (Cyan)
        if let Some((arp_note, _, _)) = self.arp_active_note {
            let idx = arp_note as i32 - tg.first_note as i32;
            if idx >= 0 && (idx as usize) < colors.len() {
                colors[idx as usize] = KeyColor { r: 0, g: 0, b: 255 };
            }
        }

        tg.set_key_leds(&colors)
    }
    fn render_display(&self, tg: &mut TargetDevice) -> Result<(), KeyboardError> {
        tg.display_buffer_mut().clear();
        match self.mode {
            Mode::Performance => {
                tg.display_text(0, 0, "PERFORM", TextAlignment::Center)?;
                for knob in 0..NUM_KNOBS {
                    self.draw_knob(tg, knob);
                }
            }
            Mode::Scale => {
                let root = &tg.root_note.display_text().to_string();

                tg.display_text(0, 0, "SCALE", TextAlignment::Center)?;
                tg.display_text(1, 0, "ROOT", TextAlignment::Center)?;
                tg.display_text(1, 1, &root, TextAlignment::Center)?;
                tg.display_text(2, 0, "SCALE", TextAlignment::Center)?;

                let mut index = 2;
                for p in tg.scale.name().split(" ") {
                    tg.display_text(index, 1, &p.to_uppercase(), TextAlignment::Center)?;
                    index = index + 1;
                }
            }
            Mode::Arp => {
                tg.display_text(0, 0, "ARP", TextAlignment::Center)?;

                let arp_state_display = match self.arp_settings.active {
                    true => "ACTIVE",
                    false => "DISABLED",
                };
                tg.display_text(0, 0, "ARP", TextAlignment::Center)?;
                tg.display_text(0, 0, arp_state_display, TextAlignment::Center)?;
                tg.display_text(
                    1,
                    0,
                    &format!("{:.0}BPM", self.arp_settings.tempo),
                    TextAlignment::Center,
                )?;

                // Display 1: Rate
                // let rate = ArpRate::ALL[self.arp_rate_index].name();
                tg.display_text(2, 0, "RATE", TextAlignment::Center)?;
                tg.display_text(
                    2,
                    1,
                    self.arp_settings.rate.display(),
                    TextAlignment::Center,
                )?;

                // Display 2: Pattern
                // let pat = ArpPattern::ALL[self.arp_pattern_index].name();
                tg.display_text(3, 0, "PATTERN", TextAlignment::Center)?;
                tg.display_text(
                    3,
                    1,
                    self.arp_settings.pattern.display(),
                    TextAlignment::Center,
                )?;

                // Display 3: Octave
                tg.display_text(4, 0, "OCTAVE", TextAlignment::Center)?;
                tg.display_text(
                    4,
                    1,
                    &format!("{} OCT", self.arp_settings.octaves),
                    TextAlignment::Center,
                )?;

                // Display 4: Gate
                // let gate_pct = (self.arp_settings.gate * 100.0).round() as u32;
                //
                tg.display_text(5, 0, "GATE", TextAlignment::Center)?;
                tg.display_text(
                    5,
                    1,
                    &format!("{}%", self.arp_settings.gate),
                    TextAlignment::Center,
                )?;
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
