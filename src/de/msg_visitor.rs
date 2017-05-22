use std::io::{Read, Take};
use std::mem;
use serde::de::{DeserializeSeed, SeqAccess};

use error::{Error, ResultE};
use super::arg_visitor::ArgDeserializer;
use super::osc_type::OscType;

/// Deserializes a single message, within a packet.
#[derive(Debug)]
pub struct MsgVisitor<'a, R: Read + 'a> {
    read: &'a mut Take<R>,
    state: State,
}

/// Which part of the OSC message is being parsed
#[derive(Debug)]
enum State {
    /// Deserializing the address pattern.
    Address(String),
    /// Deserializing the typestring.
    Typestring,
    /// No more data to deserialize from this message.
    Done,
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
}



impl<'de, 'a, R> SeqAccess<'de> for MsgVisitor<'a, R>
    where R: Read + 'a
{
    type Error = Error;
    fn next_element_seed<T>(&mut self, seed: T) -> ResultE<Option<T::Value>>
        where T: DeserializeSeed<'de>
    {
        let (new_state, result) = match mem::replace(&mut self.state, State::Done) {
            // parse the address
            State::Address(address) => {
                (State::Typestring, seed.deserialize(OscType::String(address)).map(Some))
            },
            // parsed the address; now parse the args
            State::Typestring => {
                (State::Done, seed.deserialize(&mut ArgDeserializer::new(self.read)?).map(Some))
            },
            // parsed the address and the args; nothing left to do
            State::Done => {
                (State::Done, Ok(None))
            },
        };
        self.state = new_state;
        result
    }
}
