/// Deserialization

use std::fmt;
use std::fmt::Display;
use std::io::Read;
use std;
use serde::de;
use serde::de::Visitor;

struct OscDeserializer<R> {
    read: R,
}

#[derive(Debug)]
enum Error {
    /// User provided error message (via serde::de::Error::custom)
    Message(String),
}

/// Alias for a 'Result' with the error type 'serde_osc::de::Error'
type ResultE<T> = Result<T, Error>;


impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "serde_osc::de::Error")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref msg) => msg,
            _ => "Unknown serde_osc::de::Error",
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}


impl<R> de::Deserializer for OscDeserializer<R>
    where R: Read
{
    type Error = Error;
    fn deserialize<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_bool<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_u8<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_u16<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_u32<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_u64<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_i8<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_i16<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_i32<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_i64<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_f32<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_f64<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_char<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_str<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_string<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_bytes<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_byte_buf<V>(
        self,
        visitor: V
    ) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_option<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_unit<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V
    ) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V
    ) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_seq<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_seq_fixed_size<V>(
        self,
        len: usize,
        visitor: V
    ) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_tuple<V>(
        self,
        len: usize,
        visitor: V
    ) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V
    ) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_map<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V
    ) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_struct_field<V>(
        self,
        visitor: V
    ) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V
    ) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
    fn deserialize_ignored_any<V>(
        self,
        visitor: V
    ) -> ResultE<V::Value>
    where
        V: Visitor { unimplemented!() }
}
