use std::convert::TryInto;
use std::io::Cursor;
use serde::ser::{Impossible, Serialize, Serializer, SerializeSeq, SerializeStruct, SerializeTuple};

use error::{Error, ResultE};
use super::osc_writer::OscWriter;
use super::timetag_ser::TimetagSer;

/// During serialization, we can determine whether the struct (packet)
/// being serialized is a message v.s. a bundle based on the *type* of the first
/// argument written:
///   * String => the packet is a message, and the string is its address
///   * (u32, u32) => the packet is a bundle, and the (u32, u32) is its timetag
///
/// This struct serializes the first item & yields the packet type so that
/// its user can serialize the rest of the packet appropriately.
#[derive(Debug)]
pub struct PktTypeDecoder {
    output: Cursor<Vec<u8>>,
    pkt_type: PktType,
}

#[derive(Copy, Clone, Debug)]
pub enum PktType {
    Unknown,
    Msg,
    Bundle,
}

impl PktTypeDecoder {
    pub fn new() -> Self {
        Self {
            output: Cursor::new(Vec::new()),
            pkt_type: PktType::Unknown,
        }
    }
    pub fn pkt_type(&self) -> PktType {
        self.pkt_type
    }
    pub fn data(self) -> Cursor<Vec<u8>> {
        self.output
    }
}


impl<'a> Serializer for &'a mut PktTypeDecoder {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = TimetagSeqSer<'a>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeStruct = Self::SerializeSeq;
    type SerializeTupleStruct = Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Error>;
    type SerializeMap = Impossible<Self::Ok, Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Error>;

    fn serialize_str(self, value: &str) -> ResultE<Self::Ok> {
        self.output.osc_write_str(value)?;
        self.pkt_type = PktType::Msg;
        Ok(())
    }

    fn serialize_seq(
        self, 
        _size: Option<usize>
    ) -> ResultE<Self::SerializeSeq>
    {
        Ok(TimetagSeqSer{ output: self, ser: TimetagSer::new() })
    }
    fn serialize_tuple(
        self, 
        size: usize
    ) -> ResultE<Self::SerializeTuple>
    {
        self.serialize_seq(Some(size))
    }
    fn serialize_struct(
        self, 
        _: &'static str, 
        size: usize
    ) -> ResultE<Self::SerializeStruct>
    {
        self.serialize_seq(Some(size))
    }

    default_ser!{bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char
        bytes none some unit unit_struct unit_variant newtype_struct newtype_variant
        tuple_struct tuple_variant map struct_variant}
}

pub struct TimetagSeqSer<'a> {
    output: &'a mut PktTypeDecoder,
    ser: TimetagSer,
}


impl<'a> SerializeSeq for TimetagSeqSer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> ResultE<()>
        where T: Serialize
    {
        value.serialize(&mut self.ser)
    }
    fn end(self) -> ResultE<()> {
        let timetag = self.ser.try_into()?;
        self.output.output.osc_write_timetag(timetag)?;
        self.output.pkt_type = PktType::Bundle;
        Ok(())
    }
}

impl<'a> SerializeStruct for TimetagSeqSer<'a> {
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

impl<'a> SerializeTuple for TimetagSeqSer<'a> {
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
