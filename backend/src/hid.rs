//! USB-HID transport and input report parser for the S49 MK1.

use crate::{
    events::{Button, EncoderEvent, InputEvent},
    keyboard_error::DeviceError,
};
use core::fmt;
use hidapi::{HidApi, HidDevice};

/// Number of bytes of the button bitfield in the HID input report.
const BUTTON_BYTES: usize = 5;
/// Main encoder + 8 knobs.
const NUM_ENCODERS: usize = 9;
/// Startup sequence required to initialize / black out the hardware.
const STARTUP_SEQUENCE: [u8; 3] = [0xa0, 0x00, 0x00];

#[derive(Default)]
pub struct HidConnection {
    device: Option<HidDevice>,
    button_state: [u8; BUTTON_BYTES],
    encoder_state: Option<[u16; NUM_ENCODERS]>,
}

impl fmt::Debug for HidConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HidConnection")
            .field("connected", &self.device.is_some())
            .finish()
    }
}

impl HidConnection {
    pub fn is_connected(&self) -> bool {
        self.device.is_some()
    }

    pub fn open(&mut self, vendor_id: u16, product_id: u16) -> Result<(), DeviceError> {
        let api = HidApi::new().map_err(|e| DeviceError::CantConnectKeyboard(e.to_string()))?;
        let device = api
            .open(vendor_id, product_id)
            .map_err(|e| DeviceError::CantConnectKeyboard(e.to_string()))?;

        device
            .write(&STARTUP_SEQUENCE)
            .map_err(|e| DeviceError::CannotWriteData(e.to_string()))?;

        self.device = Some(device);
        self.button_state = [0; BUTTON_BYTES];
        self.encoder_state = None;
        Ok(())
    }

    pub fn write(&self, report: &[u8]) -> Result<(), DeviceError> {
        let device = self
            .device
            .as_ref()
            .ok_or(DeviceError::KeyboardNotConnected)?;
        device
            .write(report)
            .map_err(|e| DeviceError::CannotWriteData(e.to_string()))?;
        Ok(())
    }

    /// Reads at most one HID report within `timeout_ms` and appends parsed events.
    /// If not connected and `timeout_ms > 0`, sleeps for `timeout_ms`.
    pub fn poll(
        &mut self,
        timeout_ms: i32,
        first_note: &mut u8,
        events: &mut Vec<InputEvent>,
    ) -> Result<(), DeviceError> {
        if let Some(device) = &self.device {
            let mut buf = [0u8; 64];
            let n = device
                .read_timeout(&mut buf, timeout_ms)
                .map_err(|e| DeviceError::CannotReadData(e.to_string()))?;
            if n > 0 {
                self.parse_report(&buf[..n], first_note, events);
            }
        } else if timeout_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(timeout_ms as u64));
        }
        Ok(())
    }

    fn parse_report(&mut self, buf: &[u8], first_note: &mut u8, events: &mut Vec<InputEvent>) {
        // Only report 0x01 contains the button / encoder state
        if buf.first() != Some(&0x01) || buf.len() < 1 + BUTTON_BYTES {
            return;
        }

        // Buttons: bitfield in bytes 1..=5
        for byte in 0..BUTTON_BYTES {
            let new = buf[1 + byte];
            let changed = new ^ self.button_state[byte];
            if changed == 0 {
                continue;
            }
            for bit in 0..8u8 {
                if changed & (1 << bit) != 0 {
                    events.push(InputEvent::Button {
                        button: Button::from_bit(byte as u8 * 8 + bit),
                        pressed: new & (1 << bit) != 0,
                    });
                }
            }
            self.button_state[byte] = new;
        }

        // Encoders: byte 6 = main encoder (4 bit), bytes 7..=22 = 8 knobs (10 bit, LE)
        if buf.len() >= 23 {
            let mut values = [0u16; NUM_ENCODERS];
            values[0] = (buf[6] & 0x0F) as u16;
            for i in 0..8 {
                let lo = buf[7 + i * 2] as u16;
                let hi = buf[8 + i * 2] as u16;
                values[i + 1] = (lo | (hi << 8)) & 0x3FF;
            }

            if let Some(old) = self.encoder_state {
                for (index, (&new, &prev)) in values.iter().zip(old.iter()).enumerate() {
                    if new == prev {
                        continue;
                    }
                    let modulo: i32 = if index == 0 { 16 } else { 1024 };
                    let half = modulo / 2;
                    let delta = (new as i32 - prev as i32 + half).rem_euclid(modulo) - half;
                    events.push(InputEvent::Encoder(EncoderEvent {
                        index,
                        delta,
                        raw: new,
                    }));
                }
            }
            self.encoder_state = Some(values);
        }

        // Byte 37: MIDI note of the left-most key (changes with octave up/down)
        if let Some(&note) = buf.get(37) {
            if note > 0 && note < 128 {
                *first_note = note;
            }
        }
    }
}
