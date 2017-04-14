use std::io::Read;
use byteorder::{BigEndian, ReadBytesExt};
use serde::de;
use serde::de::Visitor;

use super::error::{Error, ResultE};
use super::osc_reader::OscReader;
use super::msg_visitor::MsgVisitor;

/// Deserializes an entire OSC packet, which may contain multiple messages.
/// An OSC packet consists of an int32 indicating its length, followed by
/// the packet contents: EITHER a message OR a bundle.
/// TODO: currently only parses the first packet.
pub struct PktDeserializer<R: Read> {
    reader: Option<R>,
}


impl<R> PktDeserializer<R>
    where R: Read
{
    pub fn new(read: R) -> Self {
        Self{ reader: Some(read) }
    }
}

impl<'a, R> de::Deserializer for &'a mut PktDeserializer<R>
    where R: Read
{
    type Error = Error;
    fn deserialize<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor
    {
        match self.reader.take() {
            Some(mut reader) => {
                // First, extract the length of the packet.
                let length = reader.read_i32::<BigEndian>()?;
                let mut reader = reader.take(length as u64);
                // See if packet is a bundle or a message.
                let address = reader.parse_str()?;
                let result = match address.as_str() {
                    "#bundle" => unimplemented!(),
                    _ => visitor.visit_seq(MsgVisitor::new(&mut reader, address)),
                };
                // If the consumer only handled a portion of the sequence, we still
                // need to advance the reader so as to be ready for any next message.
                // TODO: it should be possible to read any extra chars w/o allocating.
                // Tracking: https://github.com/rust-lang/rust/issues/13989
                let size = reader.limit() as usize;
                let mut extra_chars = Vec::with_capacity(size);
                extra_chars.resize(size, Default::default());
                reader.read_exact(&mut extra_chars)?;
                result
            },
            None => Err(Error::ArgMiscount),
        }
    }

    // OSC messages are strongly typed, so we don't make use of any type hints.
    // More info: https://github.com/serde-rs/serde/blob/b7d6c5d9f7b3085a4d40a446eeb95976d2337e07/serde/src/macros.rs#L106
    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct struct_field tuple enum ignored_any
    }
}
