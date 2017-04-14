use std::io::{Read, Take};
use std::vec;
use serde::de;
use serde::de::{DeserializeSeed, SeqVisitor, Visitor};

use super::error::{Error, ResultE};
use super::osc_reader::OscReader;
use super::maybeskipcomma::MaybeSkipComma;

/// Deserializes a single message, within a packet.
pub struct MsgVisitor<'a, R: Read + 'a> {
    read: &'a mut Take<R>,
    state: State,
}

/// Which part of the OSC message is being parsed
enum State {
    /// Deserializing the address pattern.
    Address,
    /// Deserializing the typestring.
    Typestring,
    /// Deserializing the argument data.
    /// Each entry in the Vec is the typecode we parsed earlier
    /// We store this as an iterator to avoid tracking the index of the current arg.
    Arguments(MaybeSkipComma<vec::IntoIter<u8>>),
}

/// Struct to deserialize a single element from the OSC message sequence.
/// (e.g. just the address, or the first argument, etc).
enum OscType {
    I32(i32),
    F32(f32),
    String(String),
    Blob(Vec<u8>),
    // TODO: Bundle
}


impl<'a, R> MsgVisitor<'a, R>
    where R: Read + 'a
{
    pub fn new(read: &'a mut Take<R>) -> Self {
        Self {
            read: read,
            state: State::Address,
        }
    }

    fn parse_next(&mut self) -> ResultE<Option<OscType>> {
        let typetag = match self.state {
            State::Address => {
                let address = self.read.parse_str()?;
                // Successfully parsed the address component; advance to the typestring.
                self.state = State::Typestring;
                return Ok(Some(OscType::String(address)));
            },
            State::Typestring => {
                // parse the type tag
                let mut tags = self.read.parse_typetag()?;
                let first_tag = tags.next();
                self.state = State::Arguments(tags);
                first_tag
            },
            State::Arguments(ref mut tags) => {
                // Because parse_arg borrows self as mut, we need to do this weird
                // thing where we pop the typetag here, and then call parse_arg OUTSIDE
                tags.next()
            },
        };
        match typetag {
            None => Ok(None),
            Some(tag) => self.parse_arg(tag).map(|arg| Some(arg))
        }
    }
    fn parse_arg(&mut self, typecode: u8) -> ResultE<OscType> {
        match typecode {
            b'i' => self.read.parse_i32().map(|i| { OscType::I32(i) }),
            b'f' => self.read.parse_f32().map(|f| { OscType::F32(f) }),
            b's' => self.read.parse_str().map(|s| { OscType::String(s) }),
            b'b' => self.read.parse_blob().map(|b| { OscType::Blob(b) }),
            c => Err(Error::UnknownType(c)),
        }
    }
}



impl de::Deserializer for OscType {
    type Error = Error;
    // deserializes a single item from the message, consuming self.
    fn deserialize<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor
    {
        match self {
            OscType::I32(i) => visitor.visit_i32(i),
            OscType::F32(f) => visitor.visit_f32(f),
            OscType::String(s) => visitor.visit_string(s),
            // TODO: If the user is attempting to deserialize a Vec<u8>, this
            //   will error! We should make use of the deserialize_seq function
            //   in this case.
            OscType::Blob(b) => visitor.visit_byte_buf(b),
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


impl<'a, R> SeqVisitor for MsgVisitor<'a, R>
    where R: Read + 'a
{
    type Error = Error;
    fn visit_seed<T>(&mut self, seed: T) -> ResultE<Option<T::Value>>
        where T: DeserializeSeed
    {
        // Return None when the message has been fully parsed,
        // else call seed.deserialize to deserialize the next item.
        let value = self.parse_next()?;
        match value {
            // end of sequence
            None => Ok(None),
            Some(osc_arg) => seed.deserialize(osc_arg)
                .map(|value| { Some(value) }),
        }
    }
}
