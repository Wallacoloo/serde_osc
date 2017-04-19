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

/// Serialize `value` into an OSC packet, and write the contents into `write`.
/// Note that serialization of structs is done only based on the ordering
/// of fields; their names are not preserved in the output.
pub fn to_write<S: ?Sized, W: Write>(write: &mut W, value: &S) -> ResultE<()>
    where W: Write, S: serde::ser::Serialize
{
    let mut ser = Serializer::new(write.by_ref());
    value.serialize(&mut ser)
}

/// Serializes `value` into a `Vec<u8>` type.
/// This is a wrapper around the `to_write` function.
pub fn to_vec<T: ?Sized>(value: &T) -> ResultE<Vec<u8>>
    where T: serde::ser::Serialize
{
    let mut output = Cursor::new(Vec::new());
    to_write(&mut output, value)?;
    Ok(output.into_inner())
}
