use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

use backend::{models::keycolor::KeyColor, targetdevice::TargetDevice};

/// All keys pressed in this timespan gets merged as an chord
const CHORD_CAPTURE_WINDOW: Duration = Duration::from_millis(15);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Left,
    Right,
    Down,
    Up,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TickOutcome {
    pub leds_dirty: bool,
    pub display_dirty: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ActiveArpNote {
    pub note: u8,
    expires_at: Instant,
}

impl ActiveArpNote {
    pub fn new(note: u8, started_at: Instant, gate: Duration) -> Self {
        Self {
            note,
            expires_at: started_at + gate,
        }
    }

    pub fn is_expired(self, now: Instant) -> bool {
        now >= self.expires_at
    }
}

/// Musical parameters of the arpeggiator (staged while playing, applied at pattern end).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArpParams {
    pub tempo: f32,
    pub rate: ArpRate,
    pub pattern: ArpPattern,
    pub octaves: usize,
    pub gate: usize,
}

impl Default for ArpParams {
    fn default() -> Self {
        Self {
            tempo: 120.0,
            rate: ArpRate::default(),
            pattern: ArpPattern::default(),
            octaves: 1,
            gate: 100,
        }
    }
}

impl ArpParams {
    pub fn step_duration(&self) -> Duration {
        let beat_secs = 60.0 / self.tempo.max(20.0);
        Duration::from_secs_f32(beat_secs * self.rate.factor())
    }
}

/// Selectable entries in the hardware Arp menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArpMenuItem {
    #[default]
    Active,
    Tempo,
    Rate,
    Pattern,
    Octaves,
    Gate,
}

impl Cycle for ArpMenuItem {
    const ALL: &'static [Self] = &[
        Self::Active,
        Self::Tempo,
        Self::Rate,
        Self::Pattern,
        Self::Octaves,
        Self::Gate,
    ];
}

impl ArpMenuItem {
    pub fn title(self) -> &'static str {
        match self {
            Self::Active => "ARP",
            Self::Tempo => "TEMPO",
            Self::Rate => "RATE",
            Self::Pattern => "PATTERN",
            Self::Octaves => "OCTAVE",
            Self::Gate => "GATE",
        }
    }
}

/// Arpeggiator playback engine with pattern-quantized parameter updates.
pub struct Arpeggiator {
    pub active: bool,
    pub color: KeyColor,
    pub menu_item: ArpMenuItem,
    active_params: ArpParams,
    staged_params: ArpParams,
    step_index: usize,
    next_step_at: Option<Instant>,
    chord_arm_until: Option<Instant>,
    active_note: Option<ActiveArpNote>,
}

impl Default for Arpeggiator {
    fn default() -> Self {
        let params = ArpParams::default();
        Self {
            active: true,
            color: KeyColor { r: 0, g: 0, b: 255 },
            menu_item: ArpMenuItem::default(),
            active_params: params,
            staged_params: params,
            step_index: 0,
            next_step_at: None,
            chord_arm_until: None,
            active_note: None,
        }
    }
}

impl Arpeggiator {
    pub fn active_note(&self) -> Option<u8> {
        self.active_note.map(|n| n.note)
    }

    pub fn has_pending_changes(&self) -> bool {
        self.active_params != self.staged_params
    }

    /// Called whenever a piano key is pressed so chords always start at position 0.
    pub fn on_note_pressed(&mut self, was_empty: bool) {
        self.step_index = 0;
        if was_empty {
            self.active_params = self.staged_params;
            self.chord_arm_until = Some(Instant::now() + CHORD_CAPTURE_WINDOW);
            self.next_step_at = None;
        }
    }

    pub fn poll_timeout_ms(&self, default_ms: i32) -> i32 {
        if !self.active {
            return default_ms;
        }
        let earliest = [
            self.chord_arm_until,
            self.active_note.map(|n| n.expires_at),
            self.next_step_at,
        ]
        .into_iter()
        .flatten()
        .min();

        let Some(deadline) = earliest else {
            return default_ms;
        };
        let now = Instant::now();
        if deadline <= now {
            0
        } else {
            ((deadline - now).as_micros().div_ceil(1000) as i32).min(default_ms)
        }
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

    /// Advances the arpeggiator clock and applies staged params once the pattern finishes.
    pub fn tick(&mut self, tg: &mut TargetDevice, held_notes: &BTreeSet<u8>) -> TickOutcome {
        if !self.active || held_notes.is_empty() {
            let leds_dirty = self.stop(tg);
            let display_dirty = self.commit_staged_params();
            self.step_index = 0;
            self.next_step_at = None;
            self.chord_arm_until = None;
            return TickOutcome {
                leds_dirty,
                display_dirty,
            };
        }

        let now = Instant::now();
        let mut outcome = TickOutcome::default();

        if self.active_note.is_some_and(|n| n.is_expired(now)) {
            outcome.leds_dirty |= self.stop(tg);
        }

        if let Some(arm_until) = self.chord_arm_until {
            if now < arm_until {
                return outcome;
            }
            self.chord_arm_until = None;
        } else if let Some(next_at) = self.next_step_at {
            if now < next_at {
                return outcome;
            }
        }

        let prev_deadline = self.next_step_at.take();

        // Check if the previous pattern cycle has finished playing its last note.
        let current_notes = Self::expand_notes(held_notes, self.active_params.octaves);
        let current_cycle_len = self.active_params.pattern.cycle_len(current_notes.len());
        if current_cycle_len == 0 || self.step_index >= current_cycle_len {
            self.step_index = 0;
            if self.commit_staged_params() {
                outcome.display_dirty = true;
            }
        }

        let notes = Self::expand_notes(held_notes, self.active_params.octaves);
        if notes.is_empty() {
            return outcome;
        }
        let cycle_len = self.active_params.pattern.cycle_len(notes.len());
        if self.step_index >= cycle_len {
            self.step_index = 0;
        }

        self.stop(tg);

        let note = self.active_params.pattern.note_at(&notes, self.step_index);
        self.step_index += 1;

        for out in tg.midi_outs_mut() {
            let _ = out.note_on(0, note, 100);
        }

        let step_dur = self.active_params.step_duration();
        let gate_dur = step_dur.mul_f32(self.active_params.gate as f32 / 100.0);
        self.active_note = Some(ActiveArpNote::new(note, now, gate_dur));

        let next = prev_deadline.map_or(now + step_dur, |deadline| {
            let candidate = deadline + step_dur;
            if candidate > now {
                candidate
            } else {
                now + step_dur
            }
        });
        self.next_step_at = Some(next);
        outcome.leds_dirty = true;

        outcome
    }

    pub fn handle_menu(&mut self, action: MenuAction) {
        match action {
            MenuAction::Left => self.menu_item = self.menu_item.down(),
            MenuAction::Right => self.menu_item = self.menu_item.up(),
            MenuAction::Up | MenuAction::Down => {
                let dir = if action == MenuAction::Up { 1 } else { -1 };
                match self.menu_item {
                    ArpMenuItem::Active => {
                        self.active = !self.active;
                        if !self.active {
                            self.active_params = self.staged_params;
                            self.step_index = 0;
                            self.next_step_at = None;
                            self.chord_arm_until = None;
                        }
                    }
                    ArpMenuItem::Tempo => {
                        self.staged_params.tempo =
                            (self.staged_params.tempo + dir as f32).clamp(20.0, 300.0);
                    }
                    ArpMenuItem::Rate => {
                        self.staged_params.rate = self.staged_params.rate.step(dir);
                    }
                    ArpMenuItem::Pattern => {
                        self.staged_params.pattern = self.staged_params.pattern.step(dir);
                    }
                    ArpMenuItem::Octaves => {
                        self.staged_params.octaves =
                            (self.staged_params.octaves as i32 + dir).clamp(1, 4) as usize;
                    }
                    ArpMenuItem::Gate => {
                        self.staged_params.gate =
                            (self.staged_params.gate as i32 + dir).clamp(1, 100) as usize;
                    }
                }

                if !self.is_playing() {
                    self.active_params = self.staged_params;
                }
            }
        }
    }

    pub fn menu_label(&self, item: ArpMenuItem) -> String {
        if self.menu_item == item {
            format!(">{}", item.title())
        } else {
            item.title().to_string()
        }
    }

    pub fn menu_value(&self, item: ArpMenuItem) -> String {
        let s = &self.staged_params;
        let a = &self.active_params;
        let mark = |changed: bool| if changed { "*" } else { "" };

        match item {
            ArpMenuItem::Active => {
                if self.active {
                    "ACTIVE".to_string()
                } else {
                    "DISABLED".to_string()
                }
            }
            ArpMenuItem::Tempo => format!("{}{:.0}BPM", mark(s.tempo != a.tempo), s.tempo),
            ArpMenuItem::Rate => format!("{}{}", mark(s.rate != a.rate), s.rate.display()),
            ArpMenuItem::Pattern => {
                format!("{}{}", mark(s.pattern != a.pattern), s.pattern.display())
            }
            ArpMenuItem::Octaves => format!("{}{} OCT", mark(s.octaves != a.octaves), s.octaves),
            ArpMenuItem::Gate => format!("{}{}%", mark(s.gate != a.gate), s.gate),
        }
    }

    fn is_playing(&self) -> bool {
        self.active
            && (self.next_step_at.is_some()
                || self.chord_arm_until.is_some()
                || self.active_note.is_some())
    }

    fn commit_staged_params(&mut self) -> bool {
        if self.active_params != self.staged_params {
            self.active_params = self.staged_params;
            true
        } else {
            false
        }
    }

    fn expand_notes(base_notes: &BTreeSet<u8>, octaves: usize) -> Vec<u8> {
        let mut notes = Vec::with_capacity(base_notes.len() * octaves);
        for oct in 0..octaves {
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

    fn step(self, dir: i32) -> Self {
        if dir >= 0 {
            self.up()
        } else {
            self.down()
        }
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

    /// Number of steps in one complete pattern cycle for `note_count` expanded notes.
    pub fn cycle_len(self, note_count: usize) -> usize {
        match self {
            ArpPattern::Up | ArpPattern::Down => note_count,
            ArpPattern::UpDown => {
                if note_count <= 1 {
                    note_count
                } else {
                    (note_count - 1) * 2
                }
            }
        }
    }

    /// Returns the note at `step` (`0..cycle_len`) within ascending `notes`.
    pub fn note_at(self, notes: &[u8], step: usize) -> u8 {
        let len = notes.len();
        match self {
            ArpPattern::Up => notes[step % len],
            ArpPattern::Down => notes[len - 1 - (step % len)],
            ArpPattern::UpDown => {
                if len == 1 {
                    notes[0]
                } else {
                    let total = (len - 1) * 2;
                    let phase = step % total;
                    if phase < len {
                        notes[phase]
                    } else {
                        notes[total - phase]
                    }
                }
            }
        }
    }
}
