#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Performance,
    Scale,
}

impl Mode {
    pub fn toggled(self) -> Self {
        match self {
            Mode::Performance => Mode::Scale,
            Mode::Scale => Mode::Performance,
        }
    }
}
