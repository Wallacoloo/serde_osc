use std;
use std::fmt;
use std::fmt::Display;
use std::io;
use std::num;
use serde::ser;

/// Alias for a 'Result' with the error type 'serde_osc::de::Error'
pub type ResultE<T> = Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    /// User provided error message (via serde::de::Error::custom)
    Message(String),
    /// Attempt to serialize a type not supported by OSC 1.0,
    /// or not in the proper place (e.g. f32 can only be serialized within a message;
    /// not in the toplevel packet).
    UnsupportedType,
    /// Error encountered due to std::io::Write
    Io(io::Error),
    /// Error converting to a valid OSC data type.
    /// e.g. OSC spec uses i32 for lengths, which might not be possible to serialize for
    /// very large sequences.
    BadCast(num::TryFromIntError),
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


impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Message(ref msg) => write!(f, "Serializer Error: {}", msg),
            Error::UnsupportedType => write!(f, "Unsupported serialization type"),
            Error::Io(ref err) => err.fmt(f),
            Error::BadCast(ref err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            Error::UnsupportedType => "Unsupported serialization type",
            Error::Io(ref io_error) => io_error.description(),
            Error::BadCast(ref cast_error) => cast_error.description(),
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Io(ref io_error) => Some(io_error),
            Error::BadCast(ref cast_error) => Some(cast_error),
            _ => None,
        }
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
