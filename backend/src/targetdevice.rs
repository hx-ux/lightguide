use crate::{keycolor::KeyColor, utils::KeyboardKey};
use harmonia::{PitchClass, Scale, ScaleKind};
use hidapi::HidDevice;
use std::fmt;

#[derive(Debug)]
pub enum KeyboardError {
    CantConnectKeyboard(String),
    CannotWriteData(String),
    KeyboardNotConnected,
    InvalidScale,
    InvalidNote,
}

impl fmt::Display for KeyboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyboardError::CantConnectKeyboard(details) => {
                write!(f, "Cannot connect to keyboard: {}", details)
            }
            KeyboardError::CannotWriteData(details) => {
                write!(f, "Cannot write data to keyboard: {}", details)
            }
            KeyboardError::KeyboardNotConnected => {
                write!(f, "Keyboard is not connected")
            }
            KeyboardError::InvalidScale => {
                write!(f, "The provided scale is invalid")
            }
            KeyboardError::InvalidNote => {
                write!(f, "The provided note is invalid")
            }
        }
    }
}
#[derive(Debug)]
pub struct TargetDevice {
    pub name: String,
    pub keys: usize,
    pub report_size: usize,
    pub key_leds_adress: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub keys_hightlight_color: KeyColor,
    pub keys_dim_color: KeyColor,
    pub response_color: KeyColor,
    device: Option<HidDevice>,
    pub root_note: PitchClass,
    pub scale: ScaleKind,
}

impl Default for TargetDevice {
    fn default() -> Self {
        Self {
            name: "kontrol49mk1".to_string(),
            keys: 49,
            report_size: 250,
            key_leds_adress: 0x82,
            vendor_id: 0x17cc,
            product_id: 0x1350,
            keys_hightlight_color: KeyColor::new(255, 0, 0),
            keys_dim_color: KeyColor::new(0, 0, 0),
            response_color: KeyColor::new(0, 100, 0),
            device: None,
            root_note: PitchClass::C,
            scale: ScaleKind::Ionian,
        }
    }
}

impl TargetDevice {
    pub fn new(
        name: String,
        keys: usize,
        report_size: usize,
        key_leds_adress: u8,
        vendor_id: u16,
        product_id: u16,
        keys_hightlight_color: KeyColor,
        keys_dim_color: KeyColor,
        response_color: KeyColor,
    ) -> Self {
        Self {
            name,
            keys,
            report_size,
            key_leds_adress,
            vendor_id,
            product_id,
            keys_hightlight_color,
            keys_dim_color,
            response_color,
            device: None,
            root_note: PitchClass::C,
            scale: ScaleKind::Ionian,
        }
    }

    /// Whether the underlying HID device is currently open.
    pub fn is_connected(&self) -> bool {
        self.device.is_some()
    }

    /// Open the HID device and store the handle inside this struct.
    pub fn open(&mut self) -> Result<(), KeyboardError> {
        let api =
            hidapi::HidApi::new().map_err(|e| KeyboardError::CantConnectKeyboard(e.to_string()))?;

        let device = api
            .open(self.vendor_id, self.product_id)
            .map_err(|e| KeyboardError::CantConnectKeyboard(e.to_string()))?;

        const STARTUP: [u8; 3] = [0xa0, 0x00, 0x00];
        device
            .write(&STARTUP)
            .map_err(|e| KeyboardError::CannotWriteData(e.to_string()))?;

        self.device = Some(device);
        Ok(())
    }

    pub fn notes_off(&self) -> Result<(), KeyboardError> {
        let device = self
            .device
            .as_ref()
            .ok_or(KeyboardError::KeyboardNotConnected)?;

        let mut report = vec![0u8; self.report_size];
        report[0] = self.key_leds_adress;
        device
            .write(&report)
            .map_err(|e| KeyboardError::CannotWriteData(e.to_string()))?;
        Ok(())
    }

    pub fn set_scale_string(&mut self, scale: String) -> Result<(), KeyboardError> {
        let s: Scale = scale.parse().map_err(|_| KeyboardError::InvalidScale)?;

        //self.root_note = scale.root;
        self.scale = s.kind;
        self.highlight_scale()?;

        Ok(())
    }

    pub fn set_scale(&mut self, root: PitchClass, scale: ScaleKind) -> Result<(), KeyboardError> {
        self.root_note = root;
        self.scale = scale;
        self.highlight_scale()?;
        Ok(())
    }

    pub fn set_root(&mut self, root: PitchClass) -> Result<(), KeyboardError> {
        self.root_note = root;
        self.highlight_scale()?;
        Ok(())
    }

    pub fn set_scale_kind(&mut self, scale: ScaleKind) -> Result<(), KeyboardError> {
        self.scale = scale;
        self.highlight_scale()?;
        Ok(())
    }

    pub fn keyboard_keys(&self) -> Vec<KeyboardKey> {
        let build_scale = Scale::new(self.root_note, self.scale);
        (0..self.keys)
            .map(|pos| {
                let pc = PitchClass::new((pos % 12) as u8);
                KeyboardKey::from_note(pc, pos, build_scale.contains(pc))
            })
            .collect()
    }

    fn highlight_scale(&self) -> Result<(), KeyboardError> {
        let device = self
            .device
            .as_ref()
            .ok_or(KeyboardError::KeyboardNotConnected)?;

        let keys = self.keyboard_keys();
        let mut report = vec![0u8; self.report_size];
        report[0] = self.key_leds_adress;

        for key in keys {
            set_key_color(
                &mut report,
                key.position,
                if key.active {
                    self.keys_hightlight_color
                } else {
                    self.keys_dim_color
                },
            );
        }
        device
            .write(&report)
            .map_err(|e| KeyboardError::CannotWriteData(e.to_string()))?;
        Ok(())
    }

    pub fn set_highlight_color(&mut self, color: KeyColor) -> Result<(), KeyboardError> {
        self.keys_hightlight_color = color;
        self.highlight_scale()?;
        Ok(())
    }

    pub fn set_dim_color(&mut self, color: KeyColor) -> Result<(), KeyboardError> {
        self.keys_dim_color = color;
        self.highlight_scale()?;
        Ok(())
    }

    pub fn read_input(&self) -> Result<(), KeyboardError> {
        let device = self
            .device
            .as_ref()
            .ok_or(KeyboardError::KeyboardNotConnected)?;
        let mut buf = [0u8; 64];
        match device.read_timeout(&mut buf, 100) {
            Ok(0) => {}
            Ok(_) => {
                // println!("RAW {:02X?}", &buf[0..16]);
                match buf[0] {
                    0x01 => {
                        let note = buf[1];
                        let vel = buf[2];
                        if vel > 0 {
                            println!("KEY {} VEL {}", note, vel);
                        } else {
                            println!("KEY_OFF {}", note);
                        }
                    }
                    0x02 => {
                        let cc = buf[1];
                        let value = buf[2];
                        match cc {
                            1 => println!("MOD {}", value),
                            7 => println!("VOL {}", value),
                            _ => println!("CC {} = {}", cc, value),
                        }
                    }
                    0x03 => {
                        let pitch = ((buf[2] as i16) << 7) | (buf[1] as i16);
                        println!("PITCH {}", pitch - 8192);
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("read error: {e}"),
        }
        Ok(())
    }
}

fn set_key_color(report: &mut [u8], key: usize, color: KeyColor) {
    let offset = 1 + key * 3;
    if offset + 3 <= report.len() {
        report[offset..offset + 3].copy_from_slice(&color.to_bytes());
    }
}
