use std::convert::TryInto;
use std::io::{Cursor, Write};
use std::mem;
use byteorder::WriteBytesExt;
use serde::ser::{Impossible, Serialize, Serializer, SerializeSeq, SerializeStruct, SerializeTuple};

use super::error::{Error, ResultE};
use super::osc_writer::OscWriter;

pub struct PktSerializer<W: Write> {
    /// Because OSC makes use of length prefixes,
    /// we have to buffer the entire output before we can write the length.
    output: W,
    state: State,
}

enum State {
    /// Before the user has called serialize_struct/serialize_seq
    Uninitialized,
    /// User has called serialize_seq (etc), and we're waiting on the next
    /// call to determine if we are a Bundle or a Message.
    ProbingPktType,
    /// We are a message: (addr+typetag, argument data)
    IsMessage(Cursor<Vec<u8>>, Cursor<Vec<u8>>),
    /// Parsing the (u32, u32) time tag of a bundle.
    ParsingTime(Option<u32>, Option<u32>),
    /// We are a bundle. The Cursor cursor owns a Vec<u8> which stores all the
    /// messages currently seen in this bundle, and the PktSerializer itself
    /// stores data related to serializing the current element in the bundle.
    IsBundle(Box<PktSerializer<Cursor<Vec<u8>>>>),
    Finalized,
}

impl<W: Write> PktSerializer<W> {
    pub fn new(output: W) -> Self {
        Self{ output, state: State::Uninitialized }
    }
    /// Write all output to the writer; disallow new output.
    /// This is necessary because the packet header depends on
    /// all the packet content
    fn finalize(&mut self) -> ResultE<()> {
        if let State::IsBundle(ref mut contents) = self.state {
            if let State::Uninitialized = contents.state {
            } else {
                // Relay the end-of sequence command to the bundle contents
                contents.finalize()?;
                if let State::Finalized = contents.state {
                    contents.state = State::Uninitialized;
                }
            }
        }
        match mem::replace(&mut self.state, State::Finalized) {
            State::IsMessage(typetag, args) => {
                // Unwrap the cursor to a Vec<u8>
                let typetag = typetag.into_inner();
                let args = args.into_inner();

                // tag needs to be null-terminated & padded to 4-byte boundary.
                let tag_pad = 4 - (typetag.len() % 4);
                let payload_size = typetag.len() + tag_pad + args.len();
                if payload_size % 4 != 0 {
                    // Sanity check; OSC requires packets to be a multiple of 4 bytes.
                    return Err(Error::BadFormat);
                }

                // Write the packet length
                self.output.osc_write_i32(payload_size.try_into()?)?;
                // Write the address and type tag
                self.output.write_all(&typetag)?;
                let zeros = b"\0\0\0\0";
                self.output.write_all(&zeros[..tag_pad])?;
                // Write the arguments
                Ok(self.output.write_all(&args)?)
            },
            State::ParsingTime(Some(first), Some(second)) => {
                // Create the actual bundle, and write the timetag
                let mut bundle_data = Cursor::new(Vec::new());
                bundle_data.osc_write_timetag((first, second))?;
                // Delegate all future calls to a new bundle object
                self.state = State::IsBundle(Box::new(
                    PktSerializer::new(bundle_data)
                ));
                Ok(())
            },
            State::IsBundle(contents) => {
                // Unwrap the bundled writer (Vec<u8>)
                let payload = contents.output.into_inner();
                let payload_size = payload.len();
                if payload_size % 4 != 0 {
                    // Sanity check; OSC requires packets to be a multiple of 4 bytes.
                    return Err(Error::BadFormat);
                }
                // Write the packet length
                self.output.osc_write_i32(payload_size.try_into()?)?;
                Ok(self.output.write_all(&payload)?)
            }
            // OSC packets must be either a message or a bundle.
            _ => Err(Error::BadFormat),
        }
    }
}

impl<'a, W: Write> Serializer for &'a mut PktSerializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Compound<'a, W>;
    type SerializeTupleStruct = Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Error>;
    type SerializeMap = Impossible<Self::Ok, Error>;
    type SerializeStruct = Self::SerializeSeq;
    type SerializeStructVariant = Impossible<Self::Ok, Error>;

    fn serialize_bool(self, _: bool) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_i8(self, _: i8) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_i16(self, _: i16) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_i32(self, value: i32) -> ResultE<Self::Ok> {
        match self.state {
            State::IsMessage(ref mut typetag, ref mut args) => {
                typetag.write_i32_tag()?;
                args.osc_write_i32(value)?;
                Ok(())
            },
            // Relay the data to the bundle element
            State::IsBundle(ref mut pkt) => pkt.serialize_i32(value),
            _ => Err(Error::UnsupportedType),
        }
    }
    fn serialize_i64(self, _: i64) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_u8(self, _: u8) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_u16(self, _: u16) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_u32(self, value: u32) -> ResultE<Self::Ok> {
        let new_state = match self.state {
            // All possible time tags are encodable
            State::ParsingTime(None, None) => Ok(Some(State::ParsingTime(Some(value), None))),
            State::ParsingTime(Some(first), None) => Ok(Some(State::ParsingTime(Some(first), Some(value)))),
            // Relay to bundle contents
            State::IsBundle(ref mut contents) => {
                contents.serialize_u32(value)?;
                Ok(None)
            },
            _ => Err(Error::UnsupportedType),
        }?;
        if let Some(new_state) = new_state {
            self.state = new_state;
        }
        Ok(())
    }
    fn serialize_u64(self, _: u64) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_f32(self, value: f32) -> ResultE<Self::Ok> {
        match self.state {
            State::IsMessage(ref mut typetag, ref mut args) => {
                typetag.write_f32_tag()?;
                args.osc_write_f32(value)?;
                Ok(())
            },
            // Relay the data to the bundle element
            State::IsBundle(ref mut pkt) => pkt.serialize_f32(value),
            _ => Err(Error::UnsupportedType),
        }
    }
    fn serialize_f64(self, _: f64) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_char(self, _: char) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_str(self, value: &str) -> ResultE<Self::Ok> {
        match self.state {
            // If the first component of this packet is a string,
            // it must be the address. Only messages have addresses.
            State::ProbingPktType => {
                assert!(value != "#bundle");
                let mut addr_typetag = Cursor::new(Vec::new());
                addr_typetag.osc_write_str(value)?;
                // the type tag start is denoted by a comma.
                addr_typetag.write_u8(b',')?;
                // add necessary padding
                self.state = State::IsMessage(addr_typetag, Cursor::new(vec![]));
                Ok(())
            },
            State::IsMessage(ref mut typetag, ref mut args) => {
                typetag.write_str_tag()?;
                args.osc_write_str(value)?;
                Ok(())
            },
            // Relay the data to the bundle element
            State::IsBundle(ref mut pkt) => pkt.serialize_str(value),
            _ => Err(Error::UnsupportedType),
        }
    }
    fn serialize_bytes(self, value: &[u8]) -> ResultE<Self::Ok> {
        match self.state {
            State::IsMessage(ref mut typetag, ref mut args) => {
                typetag.write_blob_tag()?;
                args.osc_write_blob(value)?;
                Ok(())
            },
            // Relay the data to the bundle element
            State::IsBundle(ref mut pkt) => pkt.serialize_bytes(value),
            _ => Err(Error::UnsupportedType),
        }
    }
    fn serialize_none(self) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_some<T: ?Sized + Serialize>(
        self, 
        _: &T
    ) -> ResultE<Self::Ok>
    {
        Err(Error::UnsupportedType)
    }
    fn serialize_unit(self) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_unit_struct(
        self, 
        _: &'static str
    ) -> ResultE<Self::Ok>
    {
        Err(Error::UnsupportedType)
    }
    fn serialize_unit_variant(
        self, 
        _: &'static str, 
        _: usize, 
        _: &'static str
    ) -> ResultE<Self::Ok>
    {
        Err(Error::UnsupportedType)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self, 
        _: &'static str, 
        _: &T
    ) -> ResultE<Self::Ok>
    {
        Err(Error::UnsupportedType)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self, 
        _: &'static str, 
        _: usize, 
        _: &'static str, 
        _: &T
    ) -> ResultE<Self::Ok>
    {
        Err(Error::UnsupportedType)
    }
    fn serialize_seq(
        self, 
        _size: Option<usize>
    ) -> ResultE<Self::SerializeSeq>
    {
        let new_state = match self.state {
            // Good; all packets are sequences. Now we probe the packet type
            State::Uninitialized => Ok(Some(State::ProbingPktType)),
            // If the first element of the packet is another sequence,
            // it must be the (u32, u32) timetag, which is only packaged with bundles.
            State::ProbingPktType => Ok(Some(State::ParsingTime(None, None))),
            // Relay the data to the bundle element
            State::IsBundle(ref mut pkt) => {
                pkt.serialize_seq(_size)?;
                Ok(None)
            },
            _ => { unimplemented!(); Err(Error::UnsupportedType) },
        }?;
        if let Some(new_state) = new_state {
            self.state = new_state;
        }
        Ok(Compound{ ser: self })
    }
    fn serialize_seq_fixed_size(
        self, 
        size: usize
    ) -> ResultE<Self::SerializeSeq>
    {
        self.serialize_seq(Some(size))
    }
    fn serialize_tuple(
        self, 
        size: usize
    ) -> ResultE<Self::SerializeTuple>
    {
        self.serialize_seq(Some(size))
    }
    fn serialize_tuple_struct(
        self, 
        _: &'static str, 
        _: usize
    ) -> ResultE<Self::SerializeTupleStruct>
    {
        Err(Error::UnsupportedType)
    }
    fn serialize_tuple_variant(
        self, 
        _: &'static str, 
        _: usize, 
        _: &'static str, 
        _: usize
    ) -> ResultE<Self::SerializeTupleVariant>
    {
        Err(Error::UnsupportedType)
    }
    fn serialize_map(
        self, 
        _: Option<usize>
    ) -> ResultE<Self::SerializeMap>
    {
        Err(Error::UnsupportedType)
    }
    fn serialize_struct(
        self, 
        _: &'static str, 
        size: usize
    ) -> ResultE<Self::SerializeStruct>
    {
        self.serialize_seq(Some(size))
    }
    fn serialize_struct_variant(
        self, 
        _: &'static str, 
        _: usize, 
        _: &'static str, 
        _: usize
    ) -> ResultE<Self::SerializeStructVariant>
    {
        Err(Error::UnsupportedType)
    }
}

pub struct Compound<'a, W: Write+'a> {
    ser: &'a mut PktSerializer<W>,
}


impl<'a, W: Write + 'a> SerializeSeq for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> ResultE<()>
        where T: Serialize
    {
        // If the first element is a string, then we become a message;
        //   will accept i32, f32, blob, str args.
        // If the first element we see is a timecode (seq of u32, u32),
        //   then we become a packet.
        //   Accept the timecode, and then only sequences that in turn become
        //   PktSerializers, after that.
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> ResultE<()> {
        self.ser.finalize()
    }
}

impl<'a, W: Write + 'a> SerializeStruct for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> ResultE<()>
        where T: Serialize
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> ResultE<()> {
        SerializeSeq::end(self)
    }
}

impl<'a, W: Write + 'a> SerializeTuple for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> ResultE<()>
        where T: Serialize
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> ResultE<()> {
        SerializeSeq::end(self)
    }
}
