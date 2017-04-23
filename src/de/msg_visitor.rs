use std::io::{Read, Take};
use std::mem;
use std::vec;
use serde::de;
use serde::de::{DeserializeSeed, SeqAccess};

use error::{Error, ResultE};
use super::osc_reader::OscReader;
use super::osc_type::OscType;
use super::maybe_skip_comma::MaybeSkipComma;

/// Deserializes a single message, within a packet.
pub struct MsgVisitor<'a, R: Read + 'a> {
    read: &'a mut Take<R>,
    state: State,
}

/// Which part of the OSC message is being parsed
enum State {
    /// Deserializing the address pattern.
    Address(String),
    /// Deserializing the typestring.
    Typestring,
    /// Deserializing the argument data.
    /// Each entry in the Vec is the typecode we parsed earlier
    /// We store this as an iterator to avoid tracking the index of the current arg.
    Arguments(MaybeSkipComma<vec::IntoIter<u8>>),
}

impl<'a, R> MsgVisitor<'a, R>
    where R: Read + 'a
{
    pub fn new(read: &'a mut Take<R>, address: String) -> Self {
        Self {
            read: read,
            state: State::Address(address),
        }
    }

    fn parse_next(&mut self) -> ResultE<Option<OscType>> {
        let (new_state, osc_type) = match mem::replace(&mut self.state, State::Typestring) {
            State::Address(address) => {
                // yield the address component; advance to the typestring.
                (State::Typestring, Ok(Some(OscType::String(address.clone()))))
            },
            State::Typestring => {
                let empty_typestr = MaybeSkipComma::new(Vec::with_capacity(0).into_iter());
                // If we are at the end of our buffer, then there won't be a typestring
                // and any attempt to parse it would error.
                // But the 1.0 specs recommend typestrings be optional, so return Ok(..)
                if self.read.limit() == 0 {
                    (State::Arguments(empty_typestr), Ok(None))
                } else {
                    // parse the typestring
                    let tags = self.parse_typetag();
                    match tags {
                        // Unable to parse type tag
                        Err(err) => (State::Arguments(empty_typestr), Err(err)),
                        // Parsed: yield first argument, if it exists, else None.
                        Ok(tags) => self.pop_tag(tags),
                    }
                }
            },
            State::Arguments(tags) => self.pop_tag(tags),
        };
        self.state = new_state;
        osc_type
    }
    fn parse_typetag(&mut self) -> ResultE<MaybeSkipComma<vec::IntoIter<u8>>> {
        // The type tag is a string type, with 4-byte null padding.
        // The type tag must begin with a ","
        // Note: the 1.0 specs recommend to be robust in the case of a missing type tag string.
        self.read.read_0term_bytes().map(|bytes| MaybeSkipComma::new(bytes.into_iter()))
    }
    /// Helper for parse_next function.
    /// Pops an argument tag & attempts to parse an argument of the corresponding type.
    /// Returns both the parsed argument & the new state of the parser.
    fn pop_tag(&mut self, mut tags: MaybeSkipComma<vec::IntoIter<u8>>) -> (State, ResultE<Option<OscType>>) {
        let result = match tags.next() {
            None => Ok(None),
            Some(tag) => self.parse_arg(tag).map(|arg| Some(arg)),
        };
        (State::Arguments(tags), result)
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



impl<'de, 'a, R> SeqAccess<'de> for MsgVisitor<'a, R>
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
