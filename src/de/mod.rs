use std::{convert::TryInto, io::{Cursor, Read, Seek, SeekFrom, Write}};
use byteorder::{BigEndian, WriteBytesExt};
use serde;
use error::ResultE;

mod arg_visitor;
mod bundle_visitor;
mod iter_visitor;
mod maybe_skip_comma;
mod msg_visitor;
mod osc_reader;
mod osc_type;
mod pkt_deserializer;
mod prim_deserializer;

use crate::Framing;

pub use self::pkt_deserializer::PktDeserializer as Deserializer;

/// Deserialize an OSC packet from some readable device.
pub fn from_read<'de, D, R>(mut rd: R, framing: Framing) -> ResultE<D>
    where R: Read, D: serde::de::Deserialize<'de>
{
    match framing {
        Framing::Framed => {
            let mut de = Deserializer::new(&mut rd);
            D::deserialize(&mut de)
        },
        Framing::Unframed => {
            let mut buffer = Cursor::new(Vec::new());

            let mut tempbuffer: Vec<u8> = Vec::new();
            let buflen = rd.read_to_end(&mut tempbuffer)?;
            buffer.write_i32::<BigEndian>(buflen.try_into()?)?;
            buffer.write(tempbuffer.as_slice())?;

            let mut de = Deserializer::new(&mut buffer);
            D::deserialize(&mut de)
        }
    }
}


/// Deserialize an OSC packet from a `&[u8]` type.
/// This is a wrapper around the `from_read` function.
/// Pairs nicely with ser::to_vec, as Vec<u8> is coercable to &[u8].
pub fn from_slice<'de, T>(slice: &[u8], framing: Framing) -> ResultE<T>
    where T: serde::de::Deserialize<'de>
{
    from_read(Cursor::new(slice), framing)
}
