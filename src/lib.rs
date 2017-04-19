//! # OSC protocol
//!
//! Open Sound Control (OSC) is an open, transport-independent, message-based
//! protocol developed for communication among computers, sound synthesizers,
//! and other multimedia devices.
//!
//! It mixes human-readable API endpoints (termed "addresses") with binary-encoded
//! arguments and headers. An example OSC message is
//!
//! ```ignore
//! b"\0\0\0\x24/audio/play\0,ifb\0\0\0\0\0\0\0\x01x71\x44\x68\x00\0\0\0\x04\xDE\xAD\xBE\xEF"
//! ```
//!
//! In this example, the first 4 bytes signify the length of the message;
//! the null-terminated string "/audio/play" signifies the endpoint (which
//! component the message is intended for);
//! the ",ifb" string indicates that there are 3 arguments: one `i32`, one `f32`
//! and one binary "blob" (`u8` array). The rest of the message is the payload,
//! i.e., the values corresponding to each of these arguments.
//! The full specification can be found at [http://opensoundcontrol.org/spec-1_0].
//! Serde_osc implements version 1.0 of the spec (i.e. 'i', 'f', 's' and 'b'
//! argument types).
//!
//! # Serde_osc usage
//!
//! Generic encoding of OSC packets is intended to be done via [`serde_osc::to_write`]
//! with decoding done by [`serde_osc::from_read`]. These work with any data
//! sink/source that implements `std::io::Write` or `std::io::Read`, respectively.
//!
//! Convenience functions are also provided for some common formats; see
//! [`serde_osc::to_vec`] and [`serde_osc::from_vec`].
//!
//! [`serde_osc::to_write`]: ser/fn.to_write.html
//! [`serde_osc::from_read`]: de/fn.from_read.html
//! [`serde_osc::to_vec`]: ser/fn.to_vec.html
//! [`serde_osc::from_vec`]: de/fn.from_vec.html
//! [http://opensoundcontrol.org/spec-1_0]: http://opensoundcontrol.org/spec-1_0


#![feature(try_from)]

extern crate byteorder;
#[macro_use]
extern crate serde;

/// Errors returned upon serialization/deserialization failure.
pub mod error;
/// OSC packet deserialization framework.
pub mod de;
/// OSC packet serialization framework.
pub mod ser;

pub use de::{from_read, from_vec};
pub use ser::{to_write, to_vec};
