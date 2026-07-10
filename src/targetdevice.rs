use crate::keycolor::KeyColor;
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
        Ok(())
    }

    pub fn notes_off(&self) -> Result<(), Box<dyn Error>> {
        let device = self.device.as_ref().ok_or("device not connected")?;
        let mut report = vec![0u8; self.report_size];
        report[0] = self.key_leds_adress;
        device.write(&report)?;
        Ok(())
    }

    pub fn highlight_scale(
        &self,
        root: &str,
        name: &str,
        on_color: KeyColor,
        off_color: KeyColor,
    ) -> Result<(), Box<dyn Error>> {
        let device = self.device.as_ref().ok_or("device not connected")?;

        let root = root_semitone(root).ok_or("unknown root note")?;
        let intervals = scale_intervals(name).ok_or("unknown scale name")?;

        let mut report = vec![0u8; self.report_size];
        report[0] = self.key_leds_adress;

        for key in 0..self.keys {
            let semitone = (key + root) % 12;
            let in_scale = intervals.iter().any(|&iv| (iv + root) % 12 == semitone);
            set_key_color(
                &mut report,
                key,
                if in_scale { on_color } else { off_color },
            );
        }

        device.write(&report)?;
        Ok(())
    }

    pub fn read_input(&self) -> Result<(), Box<dyn Error>> {
        let device = self.device.as_ref().ok_or("device not connected")?;
        let mut buf = [0u8; 64];
        match device.read_timeout(&mut buf, 100) {
            Ok(0) => {}
            Ok(_) => {
                println!("RAW {:02X?}", &buf[0..16]);
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

fn scale_intervals(name: &str) -> Option<[usize; 7]> {
    match name.to_lowercase().as_str() {
        "major" => Some([0, 2, 4, 5, 7, 9, 11]),
        "minor" => Some([0, 2, 3, 5, 7, 8, 10]),
        "dorian" => Some([0, 2, 3, 5, 7, 9, 10]),
        "penta" | "pentatonic" => Some([0, 2, 4, 7, 9, 0, 0]),
        _ => None,
    }
}

fn root_semitone(root: &str) -> Option<usize> {
    match root.to_lowercase().as_str() {
        "c" => Some(0),
        "c#" | "db" => Some(1),
        "d" => Some(2),
        "d#" | "eb" => Some(3),
        "e" => Some(4),
        "f" => Some(5),
        "f#" | "gb" => Some(6),
        "g" => Some(7),
        "g#" | "ab" => Some(8),
        "a" => Some(9),
        "a#" | "bb" => Some(10),
        "b" => Some(11),
        _ => None,
    }
}
