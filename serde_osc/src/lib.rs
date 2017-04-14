#![feature(try_from)]

extern crate byteorder;
#[macro_use]
extern crate serde;


pub trait NumFields {
    fn num_fields() -> usize;
}

pub mod de;
pub mod ser;
