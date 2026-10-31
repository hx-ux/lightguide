use core::fmt;

#[derive(Debug)]
pub enum KeyboardError {
    CantConnectKeyboard(String),
    CantConnectMidi(String),
    CannotWriteData(String),
    CannotReadData(String),
    KeyboardNotConnected,
    InvalidScale,
    InvalidNote,
    SerializationError(String),
    IoError(String),
    MidiOutError,
}

impl std::error::Error for KeyboardError {}

impl From<std::io::Error> for KeyboardError {
    fn from(err: std::io::Error) -> Self {
        KeyboardError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for KeyboardError {
    fn from(err: serde_json::Error) -> Self {
        KeyboardError::SerializationError(err.to_string())
    }
}

impl fmt::Display for KeyboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyboardError::CantConnectKeyboard(details) => {
                writeln!(f, "Cannot connect to keyboard: {}", details)
            }
            KeyboardError::CantConnectMidi(details) => {
                writeln!(f, "Cannot connect to MIDI port: {}", details)
            }
            KeyboardError::CannotWriteData(details) => {
                writeln!(f, "Cannot write data to keyboard: {}", details)
            }
            KeyboardError::CannotReadData(details) => {
                writeln!(f, "Cannot read data from keyboard: {}", details)
            }
            KeyboardError::KeyboardNotConnected => {
                writeln!(f, "Keyboard is not connected")
            }
            KeyboardError::InvalidScale => {
                writeln!(f, "The provided scale is invalid")
            }
            KeyboardError::InvalidNote => {
                writeln!(f, "The provided note is invalid")
            }
            KeyboardError::SerializationError(details) => {
                writeln!(f, "Serialization error: {}", details)
            }
            KeyboardError::IoError(details) => {
                writeln!(f, "IO error: {}", details)
            }
            KeyboardError::MidiOutError => {
                writeln!(f, "Could not connect to midi port ")
            }
        }
    }
}
