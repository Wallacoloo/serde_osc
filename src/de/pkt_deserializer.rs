use std::io::Read;
use byteorder::{BigEndian, ReadBytesExt};
use serde::de;
use serde::de::Visitor;

use error::{Error, ResultE};
use super::osc_reader::OscReader;
use super::msg_visitor::MsgVisitor;
use super::bundle_visitor::BundleVisitor;

/// Deserializes an entire OSC packet or bundle element (they are syntactically identical).
/// An OSC packet consists of an `i32` indicating its length, followed by
/// the packet contents: EITHER a message OR a bundle.
///
/// This is designed to be symmetric with the [`serde_osc::ser::Serializer`] behavior,
/// so note that no "#bundle" string is emitted when decoding bundles;
/// bundles and messages are differentiated by the consumer based on whether the
/// first emitted piece of data is a String (the address of the message) or a
/// `(u32, u32)` sequence (the bundle time-tag).
///
/// See [`serde_osc::ser::Serializer`] for more info regarding valid
/// deserialization targets.
///
/// [`serde_osc::ser::Serializer`]: ../ser/struct.Serializer.html
#[derive(Debug)]
pub struct PktDeserializer<'a, R: Read + 'a> {
    reader: &'a mut R,
}

impl<'a, R> PktDeserializer<'a, R>
    where R: Read + 'a
{
    pub fn new(reader: &'a mut R) -> Self {
        Self{ reader }
    }
}

impl<'de, 'a, R> de::Deserializer<'de> for &'a mut PktDeserializer<'a, R>
    where R: Read + 'a
{
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> ResultE<V::Value>
        where V: Visitor<'de>
    {
        // First, extract the length of the packet.
        let length = self.reader.read_i32::<BigEndian>()?;
        let mut reader = self.reader.take(length as u64);
        // See if packet is a bundle or a message.
        let address = reader.parse_str()?;
        let result = match address.as_str() {
            "#bundle" => visitor.visit_seq(BundleVisitor::new(&mut reader)),
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
    }

    // This struct only deserializes sequences; ignore all type hints.
    // More info: https://github.com/serde-rs/serde/blob/b7d6c5d9f7b3085a4d40a446eeb95976d2337e07/serde/src/macros.rs#L106
    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct identifier tuple enum ignored_any
    }
}
