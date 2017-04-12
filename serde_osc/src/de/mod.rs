/// Deserialization

mod error;
mod maybeskipcomma;


use byteorder::{BigEndian, ReadBytesExt};

use std::io::Read;
use std::vec;
use serde::de;
use serde::de::Visitor;

use self::error::Error;
use self::maybeskipcomma::MaybeSkipComma;
use oscarg::OscArg;

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


/// Alias for a 'Result' with the error type 'serde_osc::de::Error'
type ResultE<T> = Result<T, Error>;

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
        let mut num_zeros = 0;
        while num_zeros == 0 {
            self.read.read_exact(&mut buf)?;
            // Copy the NON-NULL characters to the buffer.
            num_zeros = buf.iter().filter(|c| **c == 0).count();
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

    // OSC messages are strongly typed, so we don't make use of any type hints.
    // More info: https://github.com/serde-rs/serde/blob/b7d6c5d9f7b3085a4d40a446eeb95976d2337e07/serde/src/macros.rs#L106
    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct struct_field tuple enum ignored_any
    }
}

