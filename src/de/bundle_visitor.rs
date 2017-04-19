use std::io::{Read, Take};
use std::mem;
use serde::de;
use serde::de::{DeserializeSeed, SeqVisitor, Visitor};

use error::{Error, ResultE};
use super::iter_visitor::IterVisitor;
use super::osc_reader::OscReader;
use super::pkt_deserializer::PktDeserializer;
use super::prim_deserializer::PrimDeserializer;

/// Deserializes a single message, within a packet.
pub struct BundleVisitor<'a, R: Read + 'a> {
    read: &'a mut Take<R>,
    state: State,
}

/// Which part of the bundle is being parsed
enum State {
    /// Parsing the 64-bit OSC time tag
    TimeTag,
    /// Parsing the body of the bundle: OSC Bundle Elements
    Elements,
}

/// Struct to deserialize a single element from the OSC bundle
enum BundleElement<'a, R: Read + 'a> {
    TimeTag((u32, u32)),
    Packet(PktDeserializer<'a, R>),
}

impl<'a, R> BundleVisitor<'a, R>
    where R: Read + 'a
{
    pub fn new(read: &'a mut Take<R>) -> Self {
        Self {
            read: read,
            state: State::TimeTag,
        }
    }
}



impl<'a, R> de::Deserializer for BundleElement<'a, R>
    where R: Read + 'a
{
    type Error = Error;
    // deserializes a single item from the message, consuming self.
    fn deserialize<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor
    {
        match self {
            BundleElement::TimeTag((sec, frac)) =>
                visitor.visit_seq(IterVisitor([sec, frac].into_iter().cloned()
                    .map(PrimDeserializer))),
            BundleElement::Packet(mut pkt) => pkt.deserialize(visitor),
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


impl<'a, R> SeqVisitor for BundleVisitor<'a, R>
    where R: Read + 'a
{
    type Error = Error;
    fn visit_seed<T>(&mut self, seed: T) -> ResultE<Option<T::Value>>
        where T: DeserializeSeed
    {
        if self.read.limit() == 0 {
            // end of bundle
            return Ok(None);
        }
        let elem = match mem::replace(&mut self.state, State::Elements) {
            State::TimeTag => BundleElement::TimeTag(self.read.parse_timetag()?),
            State::Elements => BundleElement::Packet(PktDeserializer::new(self.read)),
        };
        seed.deserialize(elem).map(Some)
    }
}
