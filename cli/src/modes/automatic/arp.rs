use std::{
    collections::HashSet,
    time::{Duration, Instant},
};
#[derive(Debug, Clone)]
pub struct ArpSettings {
    pub active: bool,
    pub tempo: f32,
    pub pattern: ArpPattern,
    pub rate: ArpRate,
    pub octaves: usize,
    pub gate: usize,
    pub held_keys: HashSet<u8>,
    pub last_tick: Instant,
    pub arp_step_index: usize,
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
            held_keys: HashSet::new(),
            last_tick: Instant::now(),
            arp_step_index: 0,
        }
    }

    pub fn step_duration(&mut self) -> Duration {
        // let g = self.
        let beat_ms = 60_000.0 / self.tempo.max(20.0);
        // let factor = match self {
        //     ArpRate::Rate1_4 => 1.0,
        //     ArpRate::Rate1_8 => 0.5,
        //     ArpRate::Rate1_16 => 0.25,
        //     ArpRate::Rate1_32 => 0.125,
        // };

        Duration::from_secs_f32((beat_ms * self.rate.factor()) / 1000.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArpRate {
    Rate1_4,
    #[default]
    Rate1_8,
    Rate1_16,
    Rate1_32,
}

impl ArpRate {
    pub const ALL: [ArpRate; 4] = [
        ArpRate::Rate1_4,
        ArpRate::Rate1_8,
        ArpRate::Rate1_16,
        ArpRate::Rate1_32,
    ];

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

impl ArpPattern {
    pub const ALL: [ArpPattern; 3] = [ArpPattern::Up, ArpPattern::Down, ArpPattern::UpDown];

    pub fn display(self) -> &'static str {
        match self {
            ArpPattern::Up => "UP",
            ArpPattern::Down => "DOWN",
            ArpPattern::UpDown => "UP-DOWN",
        }
    }
}
