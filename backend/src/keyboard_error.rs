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
                write!(f, "Cannot connect to keyboard: {}", details)
            }
            KeyboardError::CantConnectMidi(details) => {
                write!(f, "Cannot connect to keyboard MIDI port: {}", details)
            }
            KeyboardError::CannotWriteData(details) => {
                write!(f, "Cannot write data to keyboard: {}", details)
            }
            KeyboardError::CannotReadData(details) => {
                write!(f, "Cannot read data from keyboard: {}", details)
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
            KeyboardError::SerializationError(details) => {
                write!(f, "Serialization error: {}", details)
            }
            KeyboardError::IoError(details) => {
                write!(f, "IO error: {}", details)
            }
        }
    }
}
