use serde::de::{Deserializer, Visitor};
use error::{Error, ResultE};

/// Struct to deserialize a single element from the OSC message sequence.
/// (e.g. just the address, or the first argument, etc).
pub enum OscType {
    I32(i32),
    F32(f32),
    String(String),
    Blob(Vec<u8>),
}


impl<'de> Deserializer<'de> for OscType {
    type Error = Error;
    // deserializes a single item from the message, consuming self.
    fn deserialize_any<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor<'de>
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
    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct identifier tuple enum ignored_any
    }
}


