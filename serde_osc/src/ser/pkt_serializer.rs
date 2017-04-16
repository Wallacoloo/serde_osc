use std::convert::TryInto;
use std::io::{Cursor, Write};
use std::mem;
use byteorder::WriteBytesExt;
use serde::ser::{Impossible, Serialize, Serializer, SerializeSeq, SerializeStruct};

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
    IsBundle,
    Finalized,
}

impl<W: Write> PktSerializer<W> {
    pub fn new(output: W) -> Self {
        Self{ output, state: State::Uninitialized }
    }
    /// Write all output to the writer; disallow new output.
    /// This is necessary because the packet header depends on
    /// all the packet content
    pub fn finalize(&mut self) -> ResultE<()> {
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
            State::IsBundle => unimplemented!(),
            // OSC packets must be either a message or a bundle.
            _ => Err(Error::BadFormat),
        }
    }
}

impl<'a, W: Write> Serializer for &'a mut PktSerializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Impossible<Self::Ok, Error>;
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
    fn serialize_u32(self, _: u32) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
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
            }
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
            _ => Err(Error::UnsupportedType),
        }
    }
    fn serialize_bytes(self, value: &[u8]) -> ResultE<Self::Ok> {
        match self.state {
            State::IsMessage(ref mut typetag, ref mut args) => {
                typetag.write_blob_tag()?;
                args.osc_write_blob(value)?;
                Ok(())
            }
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
        _: Option<usize>
    ) -> ResultE<Self::SerializeSeq>
    {
        match self.state {
            // Good; all packets are sequences. Now we probe the packet type
            State::Uninitialized => {
                //Ok(PktSerializer{ state: State::ProbingPktType(write.by_ref()) })
                self.state = State::ProbingPktType;
                Ok(Compound{ ser: self })
            },
            // If the first element of the packet is another sequence,
            // it must be the (u32, u32) timetag, which is only packaged with bundles.
            State::ProbingPktType => {
                //Ok(PktSerializer{ state: State::IsBundle })
                unimplemented!()
            },
            _ => Err(Error::UnsupportedType),
        }
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
        _size: usize
    ) -> ResultE<Self::SerializeTuple>
    {
        Err(Error::UnsupportedType)
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
        self.serialize_element(value)
    }

    fn end(self) -> ResultE<()> {
        SerializeSeq::end(self)
    }
}
