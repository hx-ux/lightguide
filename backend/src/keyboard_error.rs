use core::fmt;

#[derive(Debug)]
pub enum DeviceError {
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

impl std::error::Error for DeviceError {}

impl From<std::io::Error> for DeviceError {
    fn from(err: std::io::Error) -> Self {
        DeviceError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for DeviceError {
    fn from(err: serde_json::Error) -> Self {
        DeviceError::SerializationError(err.to_string())
    }
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceError::CantConnectKeyboard(details) => {
                writeln!(f, "Cannot connect to keyboard: {}", details)
            }
            DeviceError::CantConnectMidi(details) => {
                writeln!(f, "Cannot connect to MIDI port: {}", details)
            }
            DeviceError::CannotWriteData(details) => {
                writeln!(f, "Cannot write data to keyboard: {}", details)
            }
            DeviceError::CannotReadData(details) => {
                writeln!(f, "Cannot read data from keyboard: {}", details)
            }
            DeviceError::KeyboardNotConnected => {
                writeln!(f, "Keyboard is not connected")
            }
            DeviceError::InvalidScale => {
                writeln!(f, "The provided scale is invalid")
            }
            DeviceError::InvalidNote => {
                writeln!(f, "The provided note is invalid")
            }
            DeviceError::SerializationError(details) => {
                writeln!(f, "Serialization error: {}", details)
            }
            DeviceError::IoError(details) => {
                writeln!(f, "IO error: {}", details)
            }
            DeviceError::MidiOutError => {
                writeln!(f, "Could not connect to midi port ")
            }
        }
    }
}
