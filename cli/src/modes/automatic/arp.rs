use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

use backend::{models::keycolor::KeyColor, targetdevice::TargetDevice};

pub enum MenuAction {
    Left,
    Right,
    Down,
    Up,
}

/// Currently playin arpnote
#[derive(Debug, Clone, Copy)]
pub struct ActiveArpNote {
    pub note: u8,
    started_at: Instant,
    gate: Duration,
}

impl ActiveArpNote {
    pub fn new(note: u8, started_at: Instant, gate: Duration) -> Self {
        Self {
            note,
            started_at,
            gate,
        }
    }

    pub fn is_expired(self, now: Instant) -> bool {
        now.duration_since(self.started_at) >= self.gate
    }
}

/// Arpeggiator playback engine owning settings and runtime clock/note state.
pub struct Arpeggiator {
    pub settings: ArpSettings,
    last_tick: Instant,
    active_note: Option<ActiveArpNote>,
}

impl Default for Arpeggiator {
    fn default() -> Self {
        Self {
            settings: ArpSettings::default(),
            last_tick: Instant::now(),
            active_note: None,
        }
    }
}

impl Arpeggiator {
    pub fn active_note(&self) -> Option<u8> {
        self.active_note.map(|n| n.note)
    }

    /// Stops the currently sounding arp note, if any. Returns `true` if a note was stopped.
    pub fn stop(&mut self, tg: &mut TargetDevice) -> bool {
        let Some(active) = self.active_note.take() else {
            return false;
        };
        for out in tg.midi_outs_mut() {
            let _ = out.note_off(0, active.note, 64);
        }
        true
    }

    /// Advances the arpeggiator clock. Returns `true` if key LEDs should be refreshed.
    pub fn tick(&mut self, tg: &mut TargetDevice, held_notes: &BTreeSet<u8>) -> bool {
        if !self.settings.active || held_notes.is_empty() {
            return self.stop(tg);
        }

        let now = Instant::now();
        let mut leds_dirty = false;

        if self.active_note.is_some_and(|n| n.is_expired(now)) {
            leds_dirty |= self.stop(tg);
        }

        let step_dur = self.settings.step_duration();
        if now.duration_since(self.last_tick) >= step_dur {
            self.last_tick = now;
            self.stop(tg);

            let notes = self.expand_notes(held_notes);
            if notes.is_empty() {
                return leds_dirty;
            }

            let note = self
                .settings
                .pattern
                .next_note(&notes, &mut self.settings.step_index);

            for out in tg.midi_outs_mut() {
                let _ = out.note_on(0, note, 100);
            }

            let gate_dur = step_dur.mul_f32(self.settings.gate as f32 / 100.0);
            self.active_note = Some(ActiveArpNote::new(note, now, gate_dur));
            leds_dirty = true;
        }

        leds_dirty
    }

    fn expand_notes(&self, base_notes: &BTreeSet<u8>) -> Vec<u8> {
        let mut notes = Vec::with_capacity(base_notes.len() * self.settings.octaves);
        for oct in 0..self.settings.octaves {
            for &n in base_notes {
                if let Some(trans) = (oct * 12)
                    .try_into()
                    .ok()
                    .and_then(|offset: u8| n.checked_add(offset))
                    .filter(|&t| t <= 127)
                {
                    notes.push(trans);
                }
            }
        }
        notes
    }
}

pub struct ArpSettings {
    pub active: bool,
    pub tempo: f32,
    pub pattern: ArpPattern,
    pub rate: ArpRate,
    pub octaves: usize,
    pub gate: usize,
    pub step_index: usize,
    pub menu_selection: u8,
    pub color: KeyColor,
}

impl ArpSettings {
    pub fn default() -> Self {
        Self {
            active: true,
            tempo: 120.0,
            rate: ArpRate::default(),
            octaves: 1,
            gate: 100,
            pattern: ArpPattern::default(),
            step_index: 0,
            menu_selection: 0,
            color: KeyColor { r: 0, g: 0, b: 255 },
        }
    }
    pub fn menu_selection_active_prefix(&self, index: u8) -> char {
        if self.menu_selection == index {
            '>'
        } else {
            ' '
        }
    }

    pub fn step_duration(&self) -> Duration {
        let beat_ms = 60_000.0 / self.tempo.max(20.0);
        Duration::from_secs_f32((beat_ms * self.rate.factor()) / 1000.0)
    }

    pub fn change_entry(&mut self, action: MenuAction) {
        match action {
            MenuAction::Left => {
                self.menu_selection = (self.menu_selection - 1) % 2;
                print!("{}", self.menu_selection)
            }
            MenuAction::Right => {
                self.menu_selection = self.menu_selection + 1 % 2;
                print!("{}", self.menu_selection)
            }
            MenuAction::Down => {
                if self.menu_selection == 0 {
                    self.active = !self.active;
                } else if self.menu_selection == 1 {
                    self.tempo -= 1.00;
                } else if self.menu_selection == 2 {
                    self.pattern = self.pattern.down();
                } else if self.menu_selection == 3 {
                    self.rate = self.rate.down();
                } else if self.menu_selection == 4 {
                    self.octaves -= 1;
                } else if self.menu_selection == 5 {
                    self.gate -= 1;
                } else if self.menu_selection == 6 {
                    self.step_index -= 1;
                }
            }
            MenuAction::Up => {
                if (self.menu_selection == 0) {
                    self.active = !self.active;
                } else if self.menu_selection == 1 {
                    self.tempo += 1.00;
                } else if self.menu_selection == 2 {
                    self.pattern = self.pattern.up();
                } else if self.menu_selection == 3 {
                    self.rate = self.rate.up();
                } else if self.menu_selection == 4 {
                    self.octaves += 1;
                } else if self.menu_selection == 5 {
                    self.gate += 1;
                } else if self.menu_selection == 6 {
                    self.step_index += 1;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArpRate {
    Rate1_4 = 0,
    #[default]
    Rate1_8 = 1,
    Rate1_16 = 2,
    Rate1_32 = 3,
}

/// Gives an enum wrapping `up()` / `down()`. Just list the variants in `ALL`.
pub trait Cycle: Copy + PartialEq + 'static {
    const ALL: &'static [Self];

    fn up(self) -> Self {
        let i = Self::ALL.iter().position(|&v| v == self).unwrap();
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    fn down(self) -> Self {
        let i = Self::ALL.iter().position(|&v| v == self).unwrap();
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

impl Cycle for ArpRate {
    const ALL: &'static [Self] = &[Self::Rate1_4, Self::Rate1_8, Self::Rate1_16, Self::Rate1_32];
}

impl ArpRate {
    pub fn display(self) -> &'static str {
        match self {
            ArpRate::Rate1_4 => "1/4",
            ArpRate::Rate1_8 => "1/8",
            ArpRate::Rate1_16 => "1/16",
            ArpRate::Rate1_32 => "1/32",
        }
    }

    pub fn factor(self) -> f32 {
        match self {
            ArpRate::Rate1_4 => 1.0,
            ArpRate::Rate1_8 => 0.5,
            ArpRate::Rate1_16 => 0.25,
            ArpRate::Rate1_32 => 0.125,
        }
    }
}

/// Arpeggiator direction / pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArpPattern {
    #[default]
    Up,
    Down,
    UpDown,
}

impl Cycle for ArpPattern {
    const ALL: &'static [Self] = &[Self::Up, Self::Down, Self::UpDown];
}

impl ArpPattern {
    pub fn display(self) -> &'static str {
        match self {
            ArpPattern::Up => "UP",
            ArpPattern::Down => "DOWN",
            ArpPattern::UpDown => "UP-DOWN",
        }
    }

    pub fn next_note(self, notes: &[u8], step_index: &mut usize) -> u8 {
        match self {
            ArpPattern::Up => {
                let idx = *step_index % notes.len();
                *step_index = (*step_index + 1) % notes.len();
                notes[idx]
            }
            ArpPattern::Down => {
                let idx = notes.len() - 1 - (*step_index % notes.len());
                *step_index = (*step_index + 1) % notes.len();
                notes[idx]
            }
            ArpPattern::UpDown => {
                if notes.len() == 1 {
                    notes[0]
                } else {
                    let total_steps = (notes.len() - 1) * 2;
                    let phase = *step_index % total_steps;
                    *step_index = (*step_index + 1) % total_steps;
                    if phase < notes.len() {
                        notes[phase]
                    } else {
                        notes[total_steps - phase]
                    }
                }
            }
        }
    }
}
