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
    arp::{ArpMenuItem, Arpeggiator, Cycle, MenuAction},
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
    arp: Arpeggiator,
    leds_dirty: bool,
    display_dirty: bool,
}

impl AutomaticMode {
    pub fn new() -> Self {
        Self {
            mode: Mode::Performance,
            knobs: [0.0; NUM_KNOBS],
            notes: HeldNotes::default(),
            shift: false,
            last_first_note: 0,
            arp: Arpeggiator::default(),
            leds_dirty: false,
            display_dirty: false,
        }
    }

    pub fn run(&mut self, tg: &mut TargetDevice) -> Result<(), KeyboardError> {
        if !tg.is_midi_connected() {
            eprintln!("warning: MIDI not connected - keys won't light up");
        }
        self.render_all_components(tg)?;

        loop {
            let timeout = self.arp.poll_timeout_ms(5);
            let events = tg.poll(timeout)?;
            for ev in events {
                self.handle(tg, ev)?;
            }

            let outcome = self.arp.tick(tg, self.notes.active_notes());
            if outcome.display_dirty && self.mode == Mode::Arp {
                self.display_dirty = true;
            }
            if outcome.leds_dirty || tg.first_note != self.last_first_note {
                self.leds_dirty = true;
            }

            if self.leds_dirty {
                self.leds_dirty = false;
                self.render_leds(tg)?;
            }
            if self.display_dirty {
                self.display_dirty = false;
                self.render_display(tg)?;
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
            InputEvent::Slider { slider, value } => {
                // Emulate the slider midi values
                tg.send_midi_event(&InputEvent::Slider { slider, value });
                Ok(())
            }
        }
    }

    /// stored and stay lit until the pedal goes back to not active
    fn on_sustain(&mut self, tg: &mut TargetDevice, value: i32) -> Result<(), KeyboardError> {
        if !self.arp.active {
            tg.send_midi_event(&InputEvent::Slider {
                slider: Slider::Sustain,
                value,
            });
        }
        if self.notes.set_sustain(value >= 64) {
            self.leds_dirty = true;
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
            self.mark_all_dirty();
            return Ok(());
        }

        if button == Button::Arp {
            self.mode = self.mode.toggle_arp_mode();
            self.mark_all_dirty();
            return Ok(());
        }

        match self.mode {
            Mode::Performance => {}
            Mode::Scale => match button {
                Button::NavigateLeft | Button::PageLeft => self.shift_root(tg, -1),
                Button::NavigateRight | Button::PageRight => self.shift_root(tg, 1),
                Button::NavigateUp => self.shift_scale(tg, -1),
                Button::NavigateDown => self.shift_scale(tg, 1),
                _ => {}
            },
            Mode::Arp => {
                let was_active = self.arp.active;
                let action = match button {
                    Button::NavigateLeft | Button::PageLeft => Some(MenuAction::Left),
                    Button::NavigateRight | Button::PageRight => Some(MenuAction::Right),
                    Button::NavigateUp => Some(MenuAction::Up),
                    Button::NavigateDown => Some(MenuAction::Down),
                    _ => None,
                };
                if let Some(action) = action {
                    self.arp.handle_menu(action);
                    self.display_dirty = true;
                }
                if !was_active && self.arp.active {
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
                } else if was_active && !self.arp.active && self.arp.stop(tg) {
                    self.leds_dirty = true;
                }
            }
        }

        Ok(())
    }

    fn on_key(&mut self, tg: &mut TargetDevice, key: KeyEvent) -> Result<(), KeyboardError> {
        let was_empty = self.notes.active_notes().is_empty();
        self.notes.handle_key(key.note, key.pressed);
        if key.pressed {
            self.arp.on_note_pressed(was_empty);
        }

        if !self.arp.active {
            tg.send_midi_event(&InputEvent::Key(key));
        }

        self.leds_dirty = true;
        Ok(())
    }

    fn on_encoder(
        &mut self,
        _tg: &mut TargetDevice,
        enc: EncoderEvent,
    ) -> Result<(), KeyboardError> {
        let Some(knob) = enc.index.checked_sub(1).filter(|&k| k < NUM_KNOBS) else {
            return Ok(());
        };

        let ticks = if self.shift {
            TICKS_PER_STEP_FINE
        } else {
            TICKS_PER_STEP
        };
        let old = self.knob_value(knob);
        self.knobs[knob] = (self.knobs[knob] + enc.delta as f32 / ticks).clamp(0.0, KNOB_MAX);

        if self.mode == Mode::Performance && self.knob_value(knob) != old {
            self.display_dirty = true;
        }

        Ok(())
    }

    fn shift_root(&mut self, tg: &mut TargetDevice, delta: i32) {
        let root = (tg.root_note.value() as i32 + delta).rem_euclid(12) as u8;
        tg.root_note = PitchClass::new(root);
        self.mark_all_dirty();
    }

    fn shift_scale(&mut self, tg: &mut TargetDevice, delta: i32) {
        let all = ScaleKind::ALL;
        let current = all.iter().position(|k| *k == tg.scale).unwrap_or(0) as i32;
        let next = (current + delta).rem_euclid(all.len() as i32) as usize;
        tg.scale = all[next];
        self.mark_all_dirty();
    }

    fn mark_all_dirty(&mut self) {
        self.display_dirty = true;
        self.leds_dirty = true;
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
        if let Some(arp_note) = self.arp.active_note() {
            let idx = arp_note as i32 - tg.first_note as i32;
            if idx >= 0 && (idx as usize) < colors.len() {
                colors[idx as usize] = self.arp.color;
            }
        }

        tg.set_key_leds(&colors)
    }

    fn render_display(&self, tg: &mut TargetDevice) -> Result<(), KeyboardError> {
        tg.display_buffer_mut().clear();
        match self.mode {
            Mode::Performance => {
                tg.display_buffer_mut()
                    .set_text(0, 0, "PERFORM", TextAlignment::Center);
                for knob in 0..NUM_KNOBS {
                    self.draw_perfomance_knob_status(tg, knob);
                }
            }
            Mode::Scale => {
                let root = tg.root_note.display_text().to_string();
                let scale_name = tg.scale.name().to_string();
                let buf = tg.display_buffer_mut();

                buf.set_text(0, 0, "SCALE", TextAlignment::Center);
                buf.set_text(1, 0, "ROOT", TextAlignment::Center);
                buf.set_text(1, 1, &root, TextAlignment::Center);
                buf.set_text(2, 0, "SCALE", TextAlignment::Center);

                for (offset, scale_part) in scale_name.split(' ').enumerate() {
                    buf.set_text(
                        2 + offset,
                        1,
                        &scale_part.to_uppercase(),
                        TextAlignment::Center,
                    );
                }
            }
            Mode::Arp => {
                let buf = tg.display_buffer_mut();
                for (display, &item) in ArpMenuItem::ALL.iter().enumerate() {
                    let label = self.arp.menu_label(item);
                    let value = self.arp.menu_value(item);
                    buf.set_text(display, 0, &label, TextAlignment::Center);
                    buf.set_text(display, 1, &value, TextAlignment::Center);
                }
                if self.arp.has_pending_changes() {
                    buf.set_text(6, 0, "QUEUED", TextAlignment::Center);
                    buf.set_text(6, 1, "WAIT END", TextAlignment::Center);
                }
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
