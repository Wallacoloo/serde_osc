use std::io::Cursor;
use serde;
use error::ResultE;

mod bundle_visitor;
mod iter_visitor;
mod maybe_skip_comma;
mod msg_visitor;
mod osc_reader;
mod pkt_deserializer;
mod prim_deserializer;

pub use self::pkt_deserializer::OwnedPktDeserializer as Deserializer;

pub fn from_vec<T>(vec: &Vec<u8>) -> ResultE<T>
    where T: serde::de::Deserialize
{
    let rd = Cursor::new(vec);
    let mut de = Deserializer::new(rd);
    T::deserialize(&mut de)
}
