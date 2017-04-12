use std;
use std::fmt;
use std::fmt::Display;
use std::io;
use serde::de;

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
        write!(f, "serde_osc::de::Error")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            _ => "Unknown serde_osc::de::Error",
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
