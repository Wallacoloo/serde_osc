use std;
use std::fmt;
use std::fmt::Display;
use std::io;
use serde::de;

/// Alias for a 'Result' with the error type 'serde_osc::de::Error'
pub type ResultE<T> = Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    /// User provided error message (via serde::de::Error::custom)
    Message(String),
    /// Unknown argument type (i.e. not a 'f'=f32, 'i'=i32, etc)
    UnknownType(u8),
    /// Attempt to read more arguments than were in the typestring
    ArgMiscount,
    /// OSC expects all data to be aligned to 4 bytes lengths.
    /// Likely violators of this are strings, especially those at the end of a packet.
    BadPadding,
    /// Error encountered due to std::io::Read
    Io(io::Error),
    /// We store ascii strings as UTF-8.
    /// Technically, this is safe, but if we received non-ascii data, we could have invalid UTF-8
    StrParseError(std::string::FromUtf8Error),
}

// Conversion from io::Error for use with the `?` operator
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}


impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Message(ref msg) => write!(f, "Deserializer Error: {}", msg),
            Error::UnknownType(typecode) => write!(f, "Unknown OSC type: '{}'", typecode as char),
            Error::ArgMiscount => write!(f, "OSC argument count mismatch"),
            Error::BadPadding => write!(f, "OSC data not padded to 4-byte boundary"),
            Error::Io(ref err) => err.fmt(f),
            Error::StrParseError(_) => write!(f, "OSC string contains illegal (non-ascii) characters"),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            Error::UnknownType(_) => "Unknown OSC typetag",
            Error::ArgMiscount => "OSC argument count mismatch",
            Error::BadPadding => "Incorrect OSC data padding",
            Error::Io(ref io_error) => io_error.description(),
            Error::StrParseError(ref utf_error) => utf_error.description(),
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Io(ref io_error) => Some(io_error),
            Error::StrParseError(ref utf_error) => Some(utf_error),
            _ => None,
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
