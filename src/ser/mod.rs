use std::io::{Cursor, Write};
use serde;
use error::ResultE;

#[macro_use]
mod serializer_defaults;

mod pkt_serializer;
mod pkt_type_decoder;
mod osc_writer;
mod timetag_ser;

pub use self::pkt_serializer::PktSerializer as Serializer;

pub fn to_vec<T: ?Sized>(value: &T) -> ResultE<Vec<u8>>
    where T: serde::ser::Serialize
{
    let mut output = Cursor::new(Vec::new());
    {
        let mut ser = Serializer::new(output.by_ref());
        let _result = value.serialize(&mut ser)?;
    }
    Ok(output.into_inner())
}
