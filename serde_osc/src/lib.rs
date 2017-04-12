extern crate byteorder;
extern crate serde;


pub trait NumFields {
    fn num_fields() -> usize;
}

pub mod de;
mod maybeskipcomma;
mod oscarg;

pub use oscarg::OscArg;
