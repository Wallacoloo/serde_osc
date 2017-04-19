use std;
use std::fmt;
use std::fmt::Display;
use std::io;
use std::num;
use std::string;
use serde::{de, ser};

/// Alias for a 'Result' with the error type 'serde_osc::de::Error'
pub type ResultE<T> = Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    /// User provided error message (via serde::de::Error::custom)
    Message(String),
    /// Unknown argument type (i.e. not a 'f'=f32, 'i'=i32, etc)
    UnsupportedType,
    /// Packet doesn't obey correct format; mismatched lengths, or
    /// attempt to read more arguments than were in the typestring (e.g.)
    BadFormat,
    /// OSC expects all data to be aligned to 4 bytes lengths.
    /// Likely violators of this are strings, especially those at the end of a packet.
    BadPadding,
    /// Error encountered due to std::io::Read
    Io(io::Error),
    /// Error converting between parsed type and what it represents.
    /// e.g. OSC spec uses i32 for lengths, which we cast to u64, but that could underflow.
    BadCast(num::TryFromIntError),
    /// We store ascii strings as UTF-8.
    /// Technically, this is safe, but if we received non-ascii data, we could have invalid UTF-8
    StrParseError(string::FromUtf8Error),
}


/// Conversion from io::Error for use with the `?` operator
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

/// Conversion from num::TryFromIntError for use with the `?` operator
impl From<num::TryFromIntError> for Error {
    fn from(e: num::TryFromIntError) -> Self {
        Error::BadCast(e)
    }
}

/// Conversion from string::FromUtf8Error for use with the `?` operator
impl From<string::FromUtf8Error> for Error {
    fn from(e: string::FromUtf8Error) -> Self {
        Error::StrParseError(e)
    }
}


impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Message(ref msg) => write!(f, "serde_osc error: {}", msg),
            Error::UnsupportedType => write!(f, "Unsupported OSC type"),
            Error::BadFormat => write!(f, "Bad OSC packet format"),
            Error::BadPadding => write!(f, "OSC data not padded to 4-byte boundary"),
            Error::Io(ref err) => err.fmt(f),
            Error::BadCast(ref err) => err.fmt(f),
            Error::StrParseError(_) => write!(f, "OSC string contains illegal (non-ascii) characters"),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            Error::UnsupportedType => "Unsupported OSC type",
            Error::BadFormat => "OSC argument count mismatch",
            Error::BadPadding => "Incorrect OSC data padding",
            Error::Io(ref io_error) => io_error.description(),
            Error::BadCast(ref cast_error) => cast_error.description(),
            Error::StrParseError(ref utf_error) => utf_error.description(),
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Io(ref io_error) => Some(io_error),
            Error::BadCast(ref cast_error) => Some(cast_error),
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
impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
