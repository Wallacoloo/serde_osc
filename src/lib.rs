#![feature(try_from)]

extern crate byteorder;
#[macro_use]
extern crate serde;

pub mod error;
pub mod de;
pub mod ser;

pub use de::{from_read, from_vec};
pub use ser::{to_write, to_vec};
