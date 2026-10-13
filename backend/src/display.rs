//! Text output for the 9 small 16-segment displays of the S49 MK1.
//! Protocol and font taken from <https://github.com/shaduzlabs/cabl> (MIT).

pub const NUM_DISPLAYS: usize = 9;
pub const CHARS_PER_LINE: usize = 8;
pub const LINES_PER_DISPLAY: usize = 2;

const BYTES_PER_ROW: usize = 16;
const ROWS: usize = 3;
const REPORT_PAYLOAD: usize = 240;
const REPORT_HEADER: [u8; 9] = [0xE0, 0x00, 0x00, 0x00, 0x00, 0x48, 0x00, 0x01, 0x00];

/// Text alignment inside one 8 character display line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// 16-segment font, ASCII 0..=127
#[rustfmt::skip]
const FONT_16SEG: [u16; 128] = [
    0x0000, 0x0578, 0x0578, 0x830f, 0x0503, 0x5a87, 0x0c7c, 0x1878, // 0..7
    0x1887, 0x4860, 0x0a81, 0x4c66, 0x5d03, 0x4a62, 0x5a7e, 0xff00, // 8..15
    0x21c0, 0x840c, 0x2466, 0x420c, 0x189f, 0xc333, 0xf878, 0x2466, // 16..23
    0xe000, 0x0700, 0x2900, 0x9400, 0x0060, 0x2466, 0xe030, 0x0703, // 24..31
    0x0000, 0x0004, 0x0204, 0x5ac0, 0x5abb, 0x7e99, 0x8b79, 0x0004, // ' ' ! " # $ % & '
    0x8400, 0x2100, 0xff00, 0x5a00, 0x2000, 0x1800, 0x0000, 0x2400, // ( ) * + , - . /
    0x24ff, 0x040c, 0x1877, 0x183f, 0x188c, 0x18bb, 0x18fb, 0x000f, // 0..7
    0x18ff, 0x18bf, 0x0021, 0x2001, 0x8103, 0x1830, 0x2403, 0x5007, // 8 9 : ; < = > ?
    0x487f, 0x18cf, 0x523f, 0x00f3, 0x423f, 0x18f3, 0x18c3, 0x10fb, // @ A..G
    0x18cc, 0x4200, 0x203c, 0x8cc0, 0x00f0, 0x05cc, 0x81cc, 0x00ff, // H..O
    0x18c7, 0x80ff, 0x98c7, 0x18bb, 0x4203, 0x00fc, 0x24c0, 0xa0cc, // P..W
    0xa500, 0x5884, 0x2433, 0x00e1, 0x8100, 0x001e, 0x0180, 0x0030, // X Y Z [ \ ] ^ _
    0x0100, 0xc860, 0x48e0, 0x0860, 0x4a60, 0x2860, 0x5a02, 0x4aa1, // ` a..g
    0x48c0, 0x4000, 0x4220, 0xc600, 0x4200, 0x5848, 0x4840, 0x4860, // h..o
    0x0ac1, 0x4a81, 0x0840, 0x9010, 0x08e0, 0x4060, 0x2040, 0xa048, // p..w
    0x6600, 0x2500, 0x2820, 0x4a12, 0x4200, 0x5221, 0x0580, 0x540c, // x y z { | } ~ DEL
];

/// Segment pattern of a character. Non ASCII characters are shown as `?`.
pub fn glyph(c: char) -> u16 {
    if c.is_ascii() {
        FONT_16SEG[c as usize]
    } else {
        FONT_16SEG[b'?' as usize]
    }
}

/// Split `text` into chunks of 8 character cells, one per display.
/// A `.` following a character stays attached to it (decimal point).
pub fn split_across_displays(text: &str) -> Vec<String> {
    let mut cells: Vec<String> = Vec::new();
    for c in text.chars() {
        match cells.last_mut() {
            Some(last) if c == '.' && !last.ends_with('.') => last.push('.'),
            _ => cells.push(c.to_string()),
        }
    }
    cells
        .chunks(CHARS_PER_LINE)
        .take(NUM_DISPLAYS)
        .map(|chunk| chunk.concat())
        .collect()
}

/// In-memory copy of what is shown on the displays.
#[derive(Debug, Clone)]
pub struct DisplayBuffer {
    data: [[u8; ROWS * BYTES_PER_ROW]; NUM_DISPLAYS],
}

impl Default for DisplayBuffer {
    fn default() -> Self {
        Self {
            data: [[0u8; ROWS * BYTES_PER_ROW]; NUM_DISPLAYS],
        }
    }
}

impl DisplayBuffer {
    /// Blank all displays.
    pub fn clear(&mut self) {
        for d in self.data.iter_mut() {
            d.fill(0);
        }
    }

    /// Write `text` into `line` (0 = top, 1 = bottom) of `display` (0..=8).
    ///
    /// A `.` directly after a character is rendered as that character's
    /// decimal point and does not use up a character cell.
    pub fn set_text(&mut self, display: usize, line: usize, text: &str, align: TextAlignment) {
        if display >= NUM_DISPLAYS || line >= LINES_PER_DISPLAY {
            return;
        }
        let row = line + 1;

        // Split into (char, has_dot) cells
        let mut cells: Vec<(char, bool)> = Vec::with_capacity(CHARS_PER_LINE);
        for c in text.chars() {
            if c == '.' {
                match cells.last_mut() {
                    Some((_, dot)) if !*dot => *dot = true,
                    _ => cells.push((' ', true)),
                }
            } else {
                cells.push((c, false));
            }
        }
        cells.truncate(CHARS_PER_LINE);

        let pad = CHARS_PER_LINE - cells.len();
        let left = match align {
            TextAlignment::Left => 0,
            TextAlignment::Center => pad / 2,
            TextAlignment::Right => pad,
        };

        let buf = &mut self.data[display];
        // clear text row and its decimal points
        buf[row * BYTES_PER_ROW..(row + 1) * BYTES_PER_ROW].fill(0);
        let dot_mask = 1u8 << (row - 1);
        for i in 0..7 {
            buf[2 * i + 1] &= !dot_mask;
        }

        for (i, (c, dot)) in cells.into_iter().enumerate() {
            let pos = left + i;
            let g = glyph(c);
            buf[row * BYTES_PER_ROW + pos * 2] = (g & 0xff) as u8;
            buf[row * BYTES_PER_ROW + pos * 2 + 1] = (g >> 8) as u8;
            // Hardware has decimal points after characters 0..=6 only
            if dot && pos < 7 {
                buf[2 * pos + 1] |= dot_mask;
            }
        }
    }

    /// Show a bar graph (0.0 ..= 1.0) in the top row of `display`.
    pub fn set_bar(&mut self, display: usize, value: f32) {
        if display >= NUM_DISPLAYS {
            return;
        }
        let steps = (value.clamp(0.0, 1.0) * 9.0).round() as u8;
        let buf = &mut self.data[display];
        let on = |n: u8| if steps > n { 0x03 } else { 0x00 };
        // 0x04 = frame around the bar
        buf[0] = 0x04 | on(0);
        for (i, idx) in [2usize, 4, 6, 8, 10, 12, 14].iter().enumerate() {
            buf[*idx] = on(i as u8 + 1);
        }
        buf[15] = on(8);
    }

    /// Remove the bar graph of `display` (keeps decimal points).
    pub fn clear_bar(&mut self, display: usize) {
        if display >= NUM_DISPLAYS {
            return;
        }
        let buf = &mut self.data[display];
        for idx in [0usize, 2, 4, 6, 8, 10, 12, 14, 15] {
            buf[idx] = 0;
        }
    }

    /// Build the 3 HID output reports (one per row) for all displays.
    pub fn reports(&self) -> [Vec<u8>; ROWS] {
        std::array::from_fn(|row| {
            let mut report = Vec::with_capacity(REPORT_HEADER.len() + REPORT_PAYLOAD);
            report.extend_from_slice(&REPORT_HEADER);
            report[3] = row as u8;
            let mut payload = [0u8; REPORT_PAYLOAD];
            for (i, d) in self.data.iter().enumerate() {
                payload[i * BYTES_PER_ROW..(i + 1) * BYTES_PER_ROW]
                    .copy_from_slice(&d[row * BYTES_PER_ROW..(row + 1) * BYTES_PER_ROW]);
            }
            report.extend_from_slice(&payload);
            report
        })
    }
}
