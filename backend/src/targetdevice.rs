use crate::keycolor::KeyColor;
use harmonia::{Note, PitchClass, Scale};
use hidapi::HidDevice;
use std::error::Error;

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
    scale: Scale,
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
            scale: Scale::new(PitchClass::C, harmonia::ScaleKind::Ionian),
        }
    }

    /// Whether the underlying HID device is currently open.
    pub fn is_connected(&self) -> bool {
        self.device.is_some()
    }

    /// Open the HID device and store the handle inside this struct.
    pub fn open(&mut self) -> Result<(), Box<dyn Error>> {
        let api = hidapi::HidApi::new()?;
        let device = api.open(self.vendor_id, self.product_id)?;
        const STARTUP: [u8; 3] = [0xa0, 0x00, 0x00];
        let _ = device.write(&STARTUP);
        self.device = Some(device);
        // Auto Highlight scale on startup
        self.highlight_scale()?;
        Ok(())
    }

    pub fn notes_off(&self) -> Result<(), Box<dyn Error>> {
        let device = self.device.as_ref().ok_or("device not connected")?;
        let mut report = vec![0u8; self.report_size];
        report[0] = self.key_leds_adress;
        device.write(&report)?;
        Ok(())
    }

    fn key_to_position(&self, note: Note) -> u8 {
        let basekey = match note.letter {
            harmonia::Letter::C => 0,
            harmonia::Letter::D => 2,
            harmonia::Letter::E => 4,
            harmonia::Letter::F => 5,
            harmonia::Letter::G => 7,
            harmonia::Letter::A => 9,
            harmonia::Letter::B => 11,
        };

        let g = basekey + note.accidental.offset() as u8;
        g
    }

    pub fn set_scale_string(&mut self, scale: String) -> Result<(), Box<dyn Error>> {
        let g_major: Scale = scale.parse().unwrap();
        self.scale = g_major;
        self.highlight_scale()?;
        Ok(())
    }

    pub fn set_scale(&mut self, scale: Scale) {
        self.scale = scale;
        let _ = self.highlight_scale();
    }

    fn highlight_scale(&self) -> Result<(), Box<dyn Error>> {
        let device = self.device.as_ref().ok_or("device not connected")?;

        match self.scale.spelled() {
            Some(s) => {
                let mut report = vec![0u8; self.report_size];
                report[0] = self.key_leds_adress;
                let allowed_keys: Vec<u8> = s
                    .into_iter()
                    .map(|single| self.key_to_position(single))
                    .collect();

                for key in 0..self.keys {
                    let pitch_class = (key % 12) as u8;
                    let in_scale = allowed_keys.contains(&pitch_class);
                    set_key_color(
                        &mut report,
                        key,
                        if in_scale {
                            self.keys_hightlight_color
                        } else {
                            self.keys_dim_color
                        },
                    );
                }

                device.write(&report)?;
            }
            None => todo!(),
        }
        Ok(())
    }

    pub fn read_input(&self) -> Result<(), Box<dyn Error>> {
        let device = self.device.as_ref().ok_or("device not connected")?;
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
