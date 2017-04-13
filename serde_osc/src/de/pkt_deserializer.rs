use std::io::Read;
use serde::de;
use serde::de::{Deserialize, DeserializeSeed, SeqVisitor, Visitor};

use super::error::{Error, ResultE};
use super::msg_visitor::MsgVisitor;

/// Deserializes an entire OSC packet, which may contain multiple messages.
/// TODO: currently only parses the first packet.
pub struct PktDeserializer<R: Read> {
    read: R,
    state: State,
}

enum State {
    Unparsed,
    Parsed,
}

impl<R> PktDeserializer<R>
    where R: Read
{
    pub fn new(read: R) -> Self {
        Self {
            read: read,
            state: State::Unparsed,
        }
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
        match self.state {
            State::Unparsed => {
                let ret = visitor.visit_seq(MsgVisitor::new(self.read.by_ref()));
                self.state = State::Parsed;
                ret
            },
            State::Parsed => Err(Error::Eof),
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
