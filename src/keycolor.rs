/// Simple RGB color for device LED control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Equivalent to the original `rgbToUint8`
    pub fn to_bytes(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

/// Static converter matching the Dart/Flutter class exactly
pub struct ColorConverter;

impl ColorConverter {
    pub fn rgb_to_uint8(r: u8, g: u8, b: u8) -> [u8; 3] {
        let red = r & 0xFF;
        let green = g & 0xFF;
        let blue = b & 0xFF;
        [red, green, blue]
    }
}
