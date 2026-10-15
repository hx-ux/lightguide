//! USB-MIDI input of the S49 MK1.
//!
//! The piano keys and the touch strips are *not* reported over HID, they are
//! sent as regular MIDI messages on the keyboard's USB-MIDI port.

use crate::events::{InputEvent, KeyEvent, Slider};
use crate::keyboard_error::KeyboardError;
use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::fmt;
use std::sync::mpsc::{channel, Receiver};

pub const DEFAULT_PORT_FILTER: &str = "komplete kontrol";

pub(crate) struct MidiLink {
    _conn: MidiInputConnection<()>,
    pub(crate) rx: Receiver<Vec<u8>>,
    pub(crate) port_name: String,
}

impl fmt::Debug for MidiLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MidiLink")
            .field("port_name", &self.port_name)
            .finish()
    }
}

/// A connected MIDI output endpoint.
pub struct MidiOut {
    conn: MidiOutputConnection,
    port_name: String,
}

impl fmt::Debug for MidiOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MidiOut")
            .field("port_name", &self.port_name)
            .finish()
    }
}

impl MidiOut {
    /// Connect to the first MIDI output port whose name contains `filter`
    /// (case-insensitive).
    pub fn connect(filter: &str) -> Result<Self, KeyboardError> {
        let midi_out = MidiOutput::new("lightguide-out")
            .map_err(|e| KeyboardError::CantConnectMidi(e.to_string()))?;

        let filter_lc = filter.to_lowercase();
        let port = midi_out
            .ports()
            .into_iter()
            .find(|p| {
                midi_out
                    .port_name(p)
                    .map(|n| n.to_lowercase().contains(&filter_lc))
                    .unwrap_or(false)
            })
            .ok_or_else(|| {
                KeyboardError::CantConnectMidi(format!("no MIDI output port matching '{filter}'"))
            })?;

        let port_name = midi_out.port_name(&port).unwrap_or_default();
        let conn = midi_out
            .connect(&port, "lightguide-out-conn")
            .map_err(|e| KeyboardError::CantConnectMidi(e.to_string()))?;

        Ok(Self { conn, port_name })
    }

    /// Send a raw MIDI byte slice.
    pub fn send_raw(&mut self, msg: &[u8]) -> Result<(), KeyboardError> {
        self.conn
            .send(msg)
            .map_err(|e| KeyboardError::CannotWriteData(e.to_string()))
    }

    /// Send a MIDI Note On message (0x90).
    pub fn note_on(&mut self, channel: u8, note: u8, velocity: u8) -> Result<(), KeyboardError> {
        self.send_raw(&[0x90 | (channel & 0x0F), note & 0x7F, velocity & 0x7F])
    }

    /// Send a MIDI Note Off message (0x80).
    pub fn note_off(&mut self, channel: u8, note: u8, velocity: u8) -> Result<(), KeyboardError> {
        self.send_raw(&[0x80 | (channel & 0x0F), note & 0x7F, velocity & 0x7F])
    }

    /// Send a Control Change (CC) message (0xB0).
    pub fn control_change(
        &mut self,
        channel: u8,
        controller: u8,
        value: u8,
    ) -> Result<(), KeyboardError> {
        self.send_raw(&[0xB0 | (channel & 0x0F), controller & 0x7F, value & 0x7F])
    }

    /// Send Program Change message (0xC0).
    pub fn program_change(&mut self, channel: u8, program: u8) -> Result<(), KeyboardError> {
        self.send_raw(&[0xC0 | (channel & 0x0F), program & 0x7F])
    }

    /// Send Channel Pressure / Aftertouch message (0xD0).
    pub fn channel_pressure(&mut self, channel: u8, pressure: u8) -> Result<(), KeyboardError> {
        self.send_raw(&[0xD0 | (channel & 0x0F), pressure & 0x7F])
    }

    /// Send Polyphonic Key Pressure message (0xA0).
    pub fn poly_pressure(
        &mut self,
        channel: u8,
        note: u8,
        pressure: u8,
    ) -> Result<(), KeyboardError> {
        self.send_raw(&[0xA0 | (channel & 0x0F), note & 0x7F, pressure & 0x7F])
    }

    /// Send Pitch Bend message (0xE0).
    ///
    /// `value` is centered at 0 (-8192..=8191), as used by [`Slider::PitchBend`].
    pub fn pitch_bend(&mut self, channel: u8, value: i32) -> Result<(), KeyboardError> {
        let raw = (value.clamp(-8192, 8191) + 8192) as u16;
        let lsb = (raw & 0x7F) as u8;
        let msb = ((raw >> 7) & 0x7F) as u8;
        self.send_raw(&[0xE0 | (channel & 0x0F), lsb, msb])
    }

    /// Send All Notes Off (CC 123) on a given channel.
    pub fn all_notes_off(&mut self, channel: u8) -> Result<(), KeyboardError> {
        self.control_change(channel, 123, 0)
    }

    /// Send All Sound Off (CC 120) on a given channel.
    pub fn all_sound_off(&mut self, channel: u8) -> Result<(), KeyboardError> {
        self.control_change(channel, 120, 0)
    }

    /// Reset All Controllers (CC 121) on a given channel.
    pub fn reset_all_controllers(&mut self, channel: u8) -> Result<(), KeyboardError> {
        self.control_change(channel, 121, 0)
    }

    /// Send an [`InputEvent`] translated directly to its corresponding MIDI message.
    ///
    /// For key events, note on/off is sent. For sliders (pitch bend, modulation, sustain, etc.),
    /// the respective CC, aftertouch, or pitch bend is transmitted.
    /// Returns `Ok(true)` if the event was translated and sent, or `Ok(false)` if the event
    /// has no direct MIDI equivalent (such as button or encoder events, unless custom mapped).
    pub fn send_input_event(&mut self, event: &InputEvent) -> Result<bool, KeyboardError> {
        match event {
            InputEvent::Key(k) => {
                if k.pressed {
                    self.note_on(k.channel, k.note, k.velocity)?;
                } else {
                    self.note_off(k.channel, k.note, k.velocity)?;
                }
                Ok(true)
            }
            InputEvent::Slider { slider, value } => match *slider {
                Slider::PitchBend => {
                    self.pitch_bend(0, *value)?;
                    Ok(true)
                }
                Slider::Modulation => {
                    self.control_change(0, 1, (*value).clamp(0, 127) as u8)?;
                    Ok(true)
                }
                Slider::Sustain => {
                    self.control_change(0, 64, (*value).clamp(0, 127) as u8)?;
                    Ok(true)
                }
                Slider::Expression => {
                    self.control_change(0, 11, (*value).clamp(0, 127) as u8)?;
                    Ok(true)
                }
                Slider::Aftertouch => {
                    self.channel_pressure(0, (*value).clamp(0, 127) as u8)?;
                    Ok(true)
                }
                Slider::ControlChange(cc) => {
                    self.control_change(0, cc, (*value).clamp(0, 127) as u8)?;
                    Ok(true)
                }
            },
            _ => Ok(false),
        }
    }
}

/// Names of all MIDI output ports visible on this system.
pub fn list_midi_out_ports() -> Result<Vec<String>, KeyboardError> {
    let midi_out = MidiOutput::new("lightguide-list-out")
        .map_err(|e| KeyboardError::CantConnectMidi(e.to_string()))?;
    Ok(midi_out
        .ports()
        .iter()
        .filter_map(|p| midi_out.port_name(p).ok())
        .collect())
}

/// Names of all MIDI input ports visible on this system (handy for debugging).
pub fn list_midi_ports() -> Result<Vec<String>, KeyboardError> {
    let midi_in = MidiInput::new("lightguide-list")
        .map_err(|e| KeyboardError::CantConnectMidi(e.to_string()))?;
    Ok(midi_in
        .ports()
        .iter()
        .filter_map(|p| midi_in.port_name(p).ok())
        .collect())
}

/// Connect to the first MIDI input port whose name contains `filter`
/// (case-insensitive).
pub(crate) fn connect() -> Result<MidiLink, KeyboardError> {
    let mut midi_in = MidiInput::new(DEFAULT_PORT_FILTER)
        .map_err(|e| KeyboardError::CantConnectMidi(e.to_string()))?;
    // We don't need sysex, clock or active sensing
    midi_in.ignore(Ignore::All);

    let filter_lc = DEFAULT_PORT_FILTER.to_lowercase();
    let port = midi_in
        .ports()
        .into_iter()
        .find(|p| {
            midi_in
                .port_name(p)
                .map(|n| n.to_lowercase().contains(&filter_lc))
                .unwrap_or(false)
        })
        .ok_or_else(|| KeyboardError::CantConnectMidi(format!("no MIDI input port matching")))?;
    let port_name = midi_in.port_name(&port).unwrap_or_default();

    let (tx, rx) = channel::<Vec<u8>>();
    let conn = midi_in
        .connect(
            &port,
            "lightguide-in",
            move |_stamp, message, _| {
                let _ = tx.send(message.to_vec());
            },
            (),
        )
        .map_err(|e| KeyboardError::CantConnectMidi(e.to_string()))?;

    Ok(MidiLink {
        _conn: conn,
        rx,
        port_name,
    })
}

/// Translate one raw MIDI message into an [`InputEvent`].
///
/// `first_note` is the MIDI note of the left-most key, used to compute
/// [`KeyEvent::key_index`].
pub(crate) fn parse_message(msg: &[u8], first_note: u8, keys: usize) -> Option<InputEvent> {
    let status = *msg.first()?;
    let kind = status & 0xF0;
    let channel = status & 0x0F;
    let d1 = msg.get(1).copied().unwrap_or(0);
    let d2 = msg.get(2).copied().unwrap_or(0);

    let key_index = |note: u8| {
        let idx = note as i32 - first_note as i32;
        (idx >= 0 && (idx as usize) < keys).then_some(idx as usize)
    };

    match kind {
        0x90 | 0x80 => {
            let pressed = kind == 0x90 && d2 > 0;
            Some(InputEvent::Key(KeyEvent {
                note: d1,
                key_index: key_index(d1),
                velocity: d2,
                pressed,
                channel,
            }))
        }
        0xB0 => {
            let slider = match d1 {
                1 => Slider::Modulation,
                11 => Slider::Expression,
                64 => Slider::Sustain,
                cc => Slider::ControlChange(cc),
            };
            Some(InputEvent::Slider {
                slider,
                value: d2 as i32,
            })
        }
        0xD0 => Some(InputEvent::Slider {
            slider: Slider::Aftertouch,
            value: d1 as i32,
        }),
        0xE0 => {
            let raw = ((d2 as i32) << 7) | d1 as i32;
            Some(InputEvent::Slider {
                slider: Slider::PitchBend,
                value: raw - 8192,
            })
        }
        _ => None,
    }
}
