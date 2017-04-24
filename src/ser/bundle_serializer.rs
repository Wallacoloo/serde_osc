use std::convert::TryInto;
use std::io::{Cursor, Write};
use serde::ser::{Impossible, Serialize, Serializer, SerializeSeq, SerializeStruct, SerializeTuple};

use error::{Error, ResultE};
use super::osc_writer::OscWriter;
use super::pkt_serializer::PktSerializer;

pub struct BundleSerializer {
    contents: Cursor<Vec<u8>>,
}
pub struct BundleElemSerializer<'a> {
    bundle: &'a mut BundleSerializer,
}

impl BundleSerializer {
    pub fn new(contents: Cursor<Vec<u8>>) -> Self {
        Self {
            contents
        }
    }
    pub fn write_into<W: Write>(self, output: &mut W) -> ResultE<()> {
        let payload = self.contents.into_inner();
        // Add 8 because we have yet to write the #bundle address
        let payload_size = 8 + payload.len();
        if payload_size % 4 != 0 {
            // Sanity check; OSC requires packets to be a multiple of 4 bytes.
            return Err(Error::BadFormat);
        }
        // Write the packet length
        output.osc_write_i32(payload_size.try_into()?)?;
        // Write the packet payload
        output.osc_write_str("#bundle")?;
        Ok(output.write_all(&payload)?)
    }
}

impl<'a> Serializer for &'a mut BundleSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = BundleElemSerializer<'a>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeStruct = Self::SerializeSeq;
    type SerializeTupleStruct = Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Error>;
    type SerializeMap = Impossible<Self::Ok, Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Error>;
    fn serialize_seq(
        self, 
        _size: Option<usize>
    ) -> ResultE<Self::SerializeSeq>
    {
        Ok(BundleElemSerializer{ bundle: self })
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

    default_ser!{bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char bytes
        str none some unit unit_struct unit_variant newtype_struct newtype_variant
        tuple_struct tuple_variant map struct_variant}
}


impl<'a> SerializeSeq for BundleElemSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<'b, T: ?Sized>(&'b mut self, value: &T) -> ResultE<()>
        where T: Serialize
    {
        // each bundle element is itself a packet.
        let mut ser = PktSerializer::new(self.bundle.contents.by_ref());
        value.serialize(&mut ser)
    }
    fn end(self) -> ResultE<()> {
        Ok(())
    }
}

impl<'a> SerializeStruct for BundleElemSerializer<'a> {
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

impl<'a> SerializeTuple for BundleElemSerializer<'a> {
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

