use std::time::{Duration, Instant};

use backend::models::keycolor::KeyColor;
pub enum MenuAction {
    Left,
    Right,
    Down,
    Up,
}

pub struct ArpSettings {
    pub active: bool,
    pub tempo: f32,
    pub pattern: ArpPattern,
    pub rate: ArpRate,
    pub octaves: usize,
    pub gate: usize,
    pub step_index: usize,
    //
    pub menu_selection: u8,
    pub last_tick: Instant,
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
            last_tick: Instant::now(),
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

    pub fn step_duration(&mut self) -> Duration {
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
}
