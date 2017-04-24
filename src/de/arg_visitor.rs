use std::io::{Read, Take};
use std::vec;
use serde::de;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};

use error::{Error, ResultE};
use super::osc_reader::OscReader;
use super::osc_type::OscType;
use super::maybe_skip_comma::MaybeSkipComma;

pub struct ArgDeserializer<'a, R: Read + 'a> {
    data: Option<ArgVisitor<'a, R>>,
}

/// Deserializes the argument data of an OSC message.
pub struct ArgVisitor<'a, R: Read + 'a> {
    read: &'a mut Take<R>,
    /// calling .next() on this returns the OSC char code of the next argument,
    /// e.g. 'i' for i32, 'f' for f32, etc.
    /// We store this as an iterator to avoid tracking the index of the current arg.
    arg_types : MaybeSkipComma<vec::IntoIter<u8>>,
}

impl<'a, R: Read + 'a> ArgDeserializer<'a, R> {
    pub fn new(read: &'a mut Take<R>) -> ResultE<Self> {
        Ok(Self {
            data: Some(ArgVisitor::new(read)?),
        })
    }
}
impl<'de, 'a, R> de::Deserializer<'de> for &'a mut ArgDeserializer<'a, R>
    where R: Read + 'a
{
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> ResultE<V::Value>
        where V: Visitor<'de>
    {
        match self.data.take() {
            Some(data) => visitor.visit_seq(data),
            // The arguments can only be deserialized once.
            None => Err(Error::BadFormat),
        }
    }

    // This struct only deserializes sequences; ignore all type hints.
    // More info: https://github.com/serde-rs/serde/blob/b7d6c5d9f7b3085a4d40a446eeb95976d2337e07/serde/src/macros.rs#L106
    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct identifier tuple enum ignored_any
    }
}


impl<'a, R> ArgVisitor<'a, R>
    where R: Read + 'a
{
    pub fn new(read: &'a mut Take<R>) -> ResultE<Self> {
        let arg_types = read.read_0term_bytes().map(|bytes| MaybeSkipComma::new(bytes.into_iter()))?;
        Ok(ArgVisitor {
            read,
            arg_types,
        })
    }
    fn parse_next(&mut self) -> ResultE<Option<OscType>> {
        match self.arg_types.next() {
            None => Ok(None),
            Some(tag) => self.parse_arg(tag).map(|arg| Some(arg)),
        }
    }
    fn parse_arg(&mut self, typecode: u8) -> ResultE<OscType> {
        match typecode {
            b'i' => self.read.parse_i32().map(|i| { OscType::I32(i) }),
            b'f' => self.read.parse_f32().map(|f| { OscType::F32(f) }),
            b's' => self.read.parse_str().map(|s| { OscType::String(s) }),
            b'b' => self.read.parse_blob().map(|b| { OscType::Blob(b) }),
            _ => Err(Error::UnsupportedType),
        }
    }
}


impl<'de, 'a, R> SeqAccess<'de> for ArgVisitor<'a, R>
    where R: Read + 'a
{
    type Error = Error;
    fn next_element_seed<T>(&mut self, seed: T) -> ResultE<Option<T::Value>>
        where T: DeserializeSeed<'de>
    {
        // Return None when the message has been fully parsed,
        // else call seed.deserialize to deserialize the next item.
        let value = self.parse_next()?;
        match value {
            // end of sequence
            None => Ok(None),
            Some(osc_arg) => seed.deserialize(osc_arg).map(Some),
        }
    }
}
