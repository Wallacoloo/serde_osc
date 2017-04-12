/// Deserialization

use byteorder::{BigEndian, ReadBytesExt};

use std;
use std::fmt;
use std::fmt::Display;
use std::io;
use std::io::Read;
use std::mem;
use std::vec;
use serde::de;
use serde::de::Visitor;

enum OscArg {
    /// 32-bit signed integer
    i(i32),
    /// 32-bit float
    f(f32),
    /// String; specified as null-terminated ascii.
    /// This might also represent the message address pattern (aka path)
    s(String),
    /// 'blob' (binary) data
    b(Vec<u8>),
}

struct MaybeSkipComma<I> {
    iter: I,
    not_first: bool,
}

struct OscDeserializer<R> {
    read: R,
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

#[derive(Debug)]
enum Error {
    /// User provided error message (via serde::de::Error::custom)
    Message(String),
    /// Unknown argument type (i.e. not a 'f'=f32, 'i'=i32, etc)
    UnknownType(u8),
    /// Attempt to read more arguments than were in the typestring
    ArgMiscount,
    /// OSC expects all data to be aligned to 4 bytes lengths.
    /// Likely violators of this are strings, especially those at the end of a packet.
    BadPadding,
    /// Error encountered due to std::io::Read
    Io(io::Error),
    /// We store ascii strings as UTF-8.
    /// Technically, this is safe, but if we received non-ascii data, we could have invalid UTF-8
    StrParseError(std::string::FromUtf8Error),
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


impl<R> OscDeserializer<R>
    where R: Read
{
    pub fn new(read: R) -> Self {
        Self {
            read: read,
            state: State::Address,
        }
    }
    /// Strings in OSC are ascii and null-terminated.
    /// Strict specification is 1-4 null terminators, to make them end on a 4-byte boundary.
    fn read_0term_bytes(&mut self) -> ResultE<Vec<u8>> {
        let mut data = Vec::new();
        // Because of the 4-byte required padding, we can process 4 characters at a time
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        while true {
            self.read.read_exact(&mut buf)?;
            // Copy the NON-NULL characters to the buffer.
            let num_zeros = buf.iter().filter(|c| **c == 0).count();
            if buf[4-num_zeros..4].iter().any(|c| *c != 0) {
                // We had data after the null terminator.
                return Err(Error::BadPadding);
            }
            data.extend_from_slice(&buf[0..4-num_zeros]);
        }
        Ok(data)
    }
    fn parse_str(&mut self) -> ResultE<String> {
        // Note: although OSC specifies ascii only, we may have data >= 128 in the vector.
        // We can safely assume a UTF-8 encoding, because no byte of any multibyte UTF-8
        // contains a zero; the only zero possible in a UTF-8 string is the ASCII zero.
        // See the UTF-8 table here: https://en.wikipedia.org/wiki/UTF-8#History
        let bytes = self.read_0term_bytes()?;
        String::from_utf8(bytes).map_err(|err| {
            Error::StrParseError(err)
        })
    }
    fn parse_typetag(&mut self) -> ResultE<MaybeSkipComma<vec::IntoIter<u8>>> {
        // The type tag is a string type, with 4-byte null padding.
        // The type tag must begin with a ","
        // Note: the 1.0 specs recommend to be robust in the case of a missing type tag string.
        self.read_0term_bytes().map(|bytes| MaybeSkipComma::new(bytes.into_iter()))
    }

    fn parse_next(&mut self) -> ResultE<OscArg> {
        let typetag = match self.state {
            State::Address => {
                let address = self.parse_str()?;
                // Successfully parsed the address component; advance to the typestring.
                self.state = State::Typestring;
                return Ok(OscArg::s(address));
            },
            State::Typestring => {
                // parse the type tag
                let mut tags = self.parse_typetag()?;
                let parsed = self.parse_arg(tags.next())?;
                self.state = State::Arguments(tags);
                return Ok(parsed);
            },
            State::Arguments(ref mut tags) => {
                // Because parse_arg borrows self as mut, we need to do this weird
                // thing where we pop the typetag here, and then call parse_arg OUTSIDE
                tags.next()
            },
        };
        let parsed = self.parse_arg(typetag)?;
        return Ok(parsed);
    }
    fn parse_arg(&mut self, typecode: Option<u8>) -> ResultE<OscArg> {
        match typecode {
            Some(b'i') => self.parse_i32().map(|i| { OscArg::i(i) }),
            Some(b'f') => self.parse_f32().map(|f| { OscArg::f(f) }),
            Some(b's') => self.parse_str().map(|s| { OscArg::s(s) }),
            Some(b'b') => self.parse_blob().map(|b| { OscArg::b(b) }),
            Some(c) => Err(Error::UnknownType(c)),
            None => Err(Error::ArgMiscount),
        }
    }
    fn parse_i32(&mut self) -> ResultE<i32> {
       Ok( self.read.read_i32::<BigEndian>()?)
    }
    fn parse_f32(&mut self) -> ResultE<f32> {
        Ok(self.read.read_f32::<BigEndian>()?)
    }
    fn parse_blob(&mut self) -> ResultE<Vec<u8>> {
        let size = self.parse_i32()?;
        // Blobs are padded to a 4-byte boundary
        let padding = ((4-size)%4) as usize;
        let padded_size = size as usize + padding;
        // Read EXACTLY this much data:
        let mut data = vec![0; padded_size];
        self.read.read_exact(&mut data)?;
        // Ensure these extra bytes where NULL (sanity check)
        if data.drain(size as usize..padded_size).any(|c| c == 0) {
            Err(Error::BadPadding)
        } else {
            Ok(data)
        }
    }
}

impl<'a, R> de::Deserializer for &'a mut OscDeserializer<R>
    where R: Read
{
    type Error = Error;
    fn deserialize<V>(self, visitor: V) -> ResultE<V::Value>
    where
        V: Visitor
    {
        let value = self.parse_next()?;
        match value {
            OscArg::i(i) => visitor.visit_i32(i),
            OscArg::f(f) => visitor.visit_f32(f),
            OscArg::s(s) => visitor.visit_string(s),
            OscArg::b(b) => visitor.visit_byte_buf(b),
        }
    }
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


// Conversion from io::Error for use with the `?` operator
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl<I> MaybeSkipComma<I> where I: Iterator<Item=u8> {
    fn new(iter: I) -> Self {
        Self {
            iter: iter,
            not_first: false,
        }
    }
    /// For the first item in the iterator: drop it if it's a comma.
    /// For all subsequent items, yield them unchanged.
    fn next(&mut self) -> Option<u8> {
        for v in self.iter.by_ref() {
            let not_first = mem::replace(&mut self.not_first, true);
            if not_first || v != b',' {
                return Some(v)
            }
        }
        None
    }
}
