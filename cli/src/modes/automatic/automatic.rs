use std::time::{Duration, Instant};

use backend::{
    display::TextAlignment,
    events::{Button, EncoderEvent, InputEvent, KeyEvent, Slider},
    extensions::pitch_extension::StringExt,
    keyboard_error::KeyboardError,
    models::keycolor::KeyColor,
    targetdevice::TargetDevice,
};
use harmonia::{PitchClass, Scale, ScaleKind};

use crate::modes::automatic::{
    arp::{ArpPattern, ArpSettings},
    held_notes::HeldNotes,
};

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
    pub fn toggle_scale_mode(self) -> Self {
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
    notes: HeldNotes,
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
            notes: HeldNotes::default(),
            shift: false,
            last_first_note: 0,
            arp_settings: ArpSettings::default(),
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
            if self.arp_active_note.is_some() {
                self.stop_arp_note(tg)?;
                self.render_leds(tg)?;
            }
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

        if self.notes.active_notes().is_empty() {
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
            let mut notes = Vec::new();
            for oct in 0..self.arp_settings.octaves {
                for &n in self.notes.active_notes() {
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
                    let idx = self.arp_settings.step_index % notes.len();
                    self.arp_settings.step_index = (self.arp_settings.step_index + 1) % notes.len();
                    notes[idx]
                }
                ArpPattern::Down => {
                    let idx = (notes.len() - 1 - (self.arp_settings.step_index % notes.len()))
                        % notes.len();
                    self.arp_settings.step_index = (self.arp_settings.step_index + 1) % notes.len();
                    notes[idx]
                }
                ArpPattern::UpDown => {
                    if notes.len() == 1 {
                        notes[0]
                    } else {
                        let total_steps = (notes.len() - 1) * 2;
                        let phase = self.arp_settings.step_index % total_steps;
                        self.arp_settings.step_index =
                            (self.arp_settings.step_index + 1) % total_steps;
                        if phase < notes.len() {
                            notes[phase]
                        } else {
                            notes[total_steps - phase]
                        }
                    }
                }
            };

            // Trigger Note On on every output
            for out in tg.midi_outs_mut() {
                let _ = out.note_on(0, note, 100);
            }

            let gate_dur = Duration::from_secs_f32(
                step_dur.as_secs_f32() * self.arp_settings.gate as f32 / 100.0,
            );
            self.arp_active_note = Some((note, now, gate_dur));

            // Visual feedback: Update LEDs so the playing arpeggiator note lights up!
            self.render_leds(tg)?;
        }

        Ok(())
    }

    fn stop_arp_note(&mut self, tg: &mut TargetDevice) -> Result<(), KeyboardError> {
        if let Some((note, _, _)) = self.arp_active_note.take() {
            for out in tg.midi_outs_mut() {
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
        if !self.arp_settings.active {
            tg.send_midi_event(&InputEvent::Slider {
                slider: Slider::Sustain,
                value,
            });
        }
        if self.notes.set_sustain(value >= 64) {
            self.render_leds(tg)?;
        }
        Ok(())
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
            self.mode = self.mode.toggle_scale_mode();
            return self.render_all_components(tg);
        }

        if button == Button::Arp {
            self.mode = self.mode.toggle_arp_mode();
            return self.render_all_components(tg);
        }

        match self.mode {
            Mode::Performance => {}
            Mode::Scale => match button {
                Button::NavigateLeft | Button::PageLeft => self.shift_root(tg, -1)?,
                Button::NavigateRight | Button::PageRight => self.shift_root(tg, 1)?,
                Button::NavigateUp => self.shift_scale(tg, -1)?,
                Button::NavigateDown => self.shift_scale(tg, 1)?,
                _ => {}
            },
            Mode::Arp => {
                let is_active = self.arp_settings.active;
                match button {
                    Button::NavigateLeft | Button::PageLeft => {
                        self.arp_settings.change_entry(super::arp::MenuAction::Left);
                        self.render_display(tg)?;
                    }

                    Button::NavigateRight | Button::PageRight => {
                        self.arp_settings
                            .change_entry(super::arp::MenuAction::Right);
                        self.render_display(tg)?;
                    }

                    Button::NavigateUp => {
                        self.arp_settings.change_entry(super::arp::MenuAction::Up);
                        self.render_display(tg)?;
                    }

                    Button::NavigateDown => {
                        self.arp_settings.change_entry(super::arp::MenuAction::Down);
                        self.render_display(tg)?;
                    }
                    _ => {}
                }
                if !is_active && self.arp_settings.active {
                    for &note in self.notes.held() {
                        for out in tg.midi_outs_mut() {
                            let _ = out.note_off(0, note, 0);
                        }
                    }
                    if self.notes.is_sustained() {
                        tg.send_midi_event(&InputEvent::Slider {
                            slider: Slider::Sustain,
                            value: 0,
                        });
                    }
                } else if is_active && !self.arp_settings.active {
                    self.stop_arp_note(tg)?;
                    self.render_leds(tg)?;
                }
            }
        }

        if self.mode == Mode::Scale {}

        Ok(())
    }

    fn on_key(&mut self, tg: &mut TargetDevice, key: KeyEvent) -> Result<(), KeyboardError> {
        self.notes.handle_key(key.note, key.pressed);

        if !self.arp_settings.active {
            tg.send_midi_event(&InputEvent::Key(key));
        }

        self.render_leds(tg)
    }

    fn on_encoder(
        &mut self,
        tg: &mut TargetDevice,
        enc: EncoderEvent,
    ) -> Result<(), KeyboardError> {
        let knob: usize = enc.index - 1;

        let ticks = if self.shift {
            TICKS_PER_STEP_FINE
        } else {
            TICKS_PER_STEP
        };
        let old = self.knob_value(knob);
        self.knobs[knob] = (self.knobs[knob] + enc.delta as f32 / ticks).clamp(0.0, KNOB_MAX);

        if self.mode == Mode::Performance && self.knob_value(knob) != old {
            self.draw_perfomance_knob_status(tg, knob);
            tg.flush_display()?;
        }

        if self.mode == Mode::Arp && self.knob_value(knob) != old {
            self.draw_perfomance_knob_status(tg, knob);
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

        for note in self.notes.lit_notes() {
            let idx = note as i32 - tg.first_note as i32;
            if idx < 0 || idx as usize >= colors.len() {
                continue;
            }
            colors[idx as usize] = tg.response_color;
        }
        // arpeggiator step color feedback
        if let Some((arp_note, _, _)) = self.arp_active_note {
            let idx = arp_note as i32 - tg.first_note as i32;
            if idx >= 0 && (idx as usize) < colors.len() {
                colors[idx as usize] = self.arp_settings.color
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
                    self.draw_perfomance_knob_status(tg, knob);
                }
            }
            Mode::Scale => {
                let root = &tg.root_note.display_text().to_string();

                tg.display_text(0, 0, "SCALE", TextAlignment::Center)?;
                tg.display_text(1, 0, "ROOT", TextAlignment::Center)?;
                tg.display_text(1, 1, &root, TextAlignment::Center)?;
                tg.display_text(2, 0, "SCALE", TextAlignment::Center)?;

                // Split scale name over 2 displays
                let mut index = 2;
                for scale_part in tg.scale.name().split(" ") {
                    tg.display_text(index, 1, &scale_part.to_uppercase(), TextAlignment::Center)?;
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
                tg.display_text(
                    0,
                    0,
                    &format!(
                        "{} {}",
                        self.arp_settings.menu_selection_active_prefix(0),
                        arp_state_display,
                    ),
                    TextAlignment::Center,
                )?;

                tg.display_text(
                    1,
                    0,
                    &format!(
                        "{} {:.0}BPM",
                        self.arp_settings.menu_selection_active_prefix(1),
                        self.arp_settings.tempo,
                    ),
                    TextAlignment::Center,
                )?;

                tg.display_text(
                    2,
                    0,
                    &format!("{} RATE", self.arp_settings.menu_selection_active_prefix(2)),
                    TextAlignment::Center,
                )?;
                tg.display_text(
                    2,
                    1,
                    self.arp_settings.rate.display(),
                    TextAlignment::Center,
                )?;

                tg.display_text(
                    3,
                    0,
                    &format!(
                        "{} PATTERN",
                        self.arp_settings.menu_selection_active_prefix(3)
                    ),
                    TextAlignment::Center,
                )?;
                tg.display_text(
                    3,
                    1,
                    self.arp_settings.pattern.display(),
                    TextAlignment::Center,
                )?;

                tg.display_text(
                    4,
                    0,
                    &format!(
                        "{} OCTAVE",
                        self.arp_settings.menu_selection_active_prefix(4)
                    ),
                    TextAlignment::Center,
                )?;
                tg.display_text(
                    4,
                    1,
                    &format!("{} OCT", self.arp_settings.octaves),
                    TextAlignment::Center,
                )?;

                tg.display_text(
                    5,
                    0,
                    &format!("{} GATE", self.arp_settings.menu_selection_active_prefix(5)),
                    TextAlignment::Center,
                )?;
                tg.display_text(
                    5,
                    1,
                    &format!("{}%", self.arp_settings.gate),
                    TextAlignment::Center,
                )?;

                tg.display_text(
                    5,
                    0,
                    &format!("{} Step", self.arp_settings.menu_selection_active_prefix(6)),
                    TextAlignment::Center,
                )?;
                tg.display_text(
                    5,
                    1,
                    &format!("{}%", self.arp_settings.step_index),
                    TextAlignment::Center,
                )?;
            }
        }
        tg.flush_display()
    }

    /// Write bar + label + value of one knob into the display buffer.
    fn draw_perfomance_knob_status(&self, tg: &mut TargetDevice, knob: usize) {
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
