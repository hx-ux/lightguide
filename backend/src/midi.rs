//! USB-MIDI input of the S49 MK1.
//!
//! The piano keys and the touch strips are *not* reported over HID, they are
//! sent as regular MIDI messages on the keyboard's USB-MIDI port.
//! Messages are received on a background thread (owned by `midir`) and
//! forwarded through a channel, so that all user callbacks are executed on the
//! thread that calls [`crate::targetdevice::TargetDevice::poll`].

use crate::events::{InputEvent, KeyEvent, Slider};
use crate::keyboard_error::KeyboardError;
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::fmt;
use std::sync::mpsc::{channel, Receiver};

/// Default (case-insensitive) substring used to find the keyboard's MIDI port.
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
pub(crate) fn connect(filter: &str) -> Result<MidiLink, KeyboardError> {
    let mut midi_in =
        MidiInput::new("lightguide").map_err(|e| KeyboardError::CantConnectMidi(e.to_string()))?;
    // We don't need sysex, clock or active sensing
    midi_in.ignore(Ignore::All);

    let filter_lc = filter.to_lowercase();
    let port = midi_in
        .ports()
        .into_iter()
        .find(|p| {
            midi_in
                .port_name(p)
                .map(|n| n.to_lowercase().contains(&filter_lc))
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            KeyboardError::CantConnectMidi(format!("no MIDI input port matching '{filter}'"))
        })?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_on_off() {
        match parse_message(&[0x90, 40, 100], 36, 49) {
            Some(InputEvent::Key(k)) => {
                assert!(k.pressed);
                assert_eq!(k.key_index, Some(4));
            }
            other => panic!("{other:?}"),
        }
        match parse_message(&[0x90, 40, 0], 36, 49) {
            Some(InputEvent::Key(k)) => assert!(!k.pressed),
            other => panic!("{other:?}"),
        }
        match parse_message(&[0x80, 20, 0], 36, 49) {
            Some(InputEvent::Key(k)) => assert_eq!(k.key_index, None),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn pitch_bend_centre() {
        assert_eq!(
            parse_message(&[0xE0, 0x00, 0x40], 36, 49),
            Some(InputEvent::Slider {
                slider: Slider::PitchBend,
                value: 0
            })
        );
    }
}
