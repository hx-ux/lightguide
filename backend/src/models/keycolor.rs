use serde::{Deserialize, Serialize};

/// Simple RGB color for device LED control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl KeyColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r & 0xFF,
            g: g & 0xFF,
            b: b & 0xFF,
        }
    }

    pub fn to_bytes(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

impl Default for KeyColor {
    fn default() -> Self {
        Self { r: 255, g: 0, b: 0 }
    }
}
