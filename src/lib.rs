#![feature(try_from)]

extern crate byteorder;
#[macro_use]
extern crate serde;

pub mod error;
pub mod de;
pub mod ser;
