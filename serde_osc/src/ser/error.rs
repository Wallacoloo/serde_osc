use std;
use std::fmt;
use std::fmt::Display;
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
}




impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Message(ref msg) => write!(f, "Serializer Error: {}", msg),
            Error::UnsupportedType => write!(f, "Unsupported serialization type"),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            Error::UnsupportedType => "Unsupported serialization type",
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            _ => None,
        }
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
