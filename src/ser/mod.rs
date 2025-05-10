use std::{convert::TryInto, io::{Cursor, Read, Seek, SeekFrom, Write}};
use serde;
use error::ResultE;

#[macro_use]
mod serializer_defaults;

mod bundle_serializer;
mod pkt_serializer;
mod pkt_type_decoder;
mod osc_writer;
mod msg_serializer;
mod timetag_ser;

use super::ser::osc_writer::OscWriter;
use crate::Framing;

pub use self::pkt_serializer::PktSerializer as Serializer;
/// Serialize `value` into an OSC packet, and write the contents into `write`.
/// Note that serialization of structs is done only based on the ordering
/// of fields; their names are not preserved in the output.
pub fn to_write<S: ?Sized, W: Write>(write: &mut W, value: &S, framing: Framing) -> ResultE<()>
    where W: Write, S: serde::ser::Serialize
{
    match framing {
        Framing::Unframed => {
            let mut ser = Serializer::new(write.by_ref());
            value.serialize(&mut ser)
        },
        Framing::Framed => {
            let mut buffer = Cursor::new(Vec::new());
            let mut ser = Serializer::new(&mut buffer);
            value.serialize(&mut ser)?;
            // get buffer length
            let buflen = buffer.seek(SeekFrom::End(0))?;
            buffer.seek(SeekFrom::Start(0))?;
            write.osc_write_i32(buflen.try_into()?)?;
            let mut tempbuf = vec![0 as u8; buflen.try_into()?];
            buffer.read(tempbuf.as_mut_slice())?;
            write.write(tempbuf.as_slice())?;
            Ok(())
        }
    }
}

/// Serializes `value` into a `Vec<u8>` type.
/// This is a wrapper around the `to_write` function.
pub fn to_vec<T: ?Sized>(value: &T, framing: Framing) -> ResultE<Vec<u8>>
    where T: serde::ser::Serialize
{
    let mut output = Cursor::new(Vec::new());
    to_write(&mut output, value, framing)?;
    Ok(output.into_inner())
}
