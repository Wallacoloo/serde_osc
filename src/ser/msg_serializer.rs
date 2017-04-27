use std::convert::TryInto;
use std::io::{Cursor, Write};
use byteorder::WriteBytesExt;
use serde::ser::{Impossible, Serialize, Serializer, SerializeSeq, SerializeStruct, SerializeTuple};

use error::{Error, ResultE};
use super::osc_writer::OscWriter;

/// Once we know we're serializing a message, we do so through this struct.
pub struct MsgSerializer {
    /// Address + typetag, merged into one field
    addr_typetag: Cursor<Vec<u8>>,
    /// Binary-formatted argument data
    args: Cursor<Vec<u8>>,
}

pub struct ArgSerializer<'a> {
    msg: &'a mut MsgSerializer,
}

impl MsgSerializer {
    pub fn new(mut address: Cursor<Vec<u8>>) -> ResultE<Self> {
        // Prepare to append type arguments in future calls
        address.write_u8(b',')?;
        Ok(Self {
            addr_typetag: address,
            args: Cursor::new(Vec::new()),
        })
    }
    pub fn write_into<W: Write>(self, output: &mut W) -> ResultE<()> {
        let typetag = self.addr_typetag.into_inner();
        let args = self.args.into_inner();
        let tag_pad = 4 - (typetag.len() % 4);
        let payload_size = typetag.len() + tag_pad + args.len();
        if payload_size % 4 != 0 {
            // Sanity check; OSC requires packets to be a multiple of 4 bytes.
            return Err(Error::BadFormat);
        }

        // Write the packet length
        output.osc_write_i32(payload_size.try_into()?)?;
        // Write the address and type tag
        output.write_all(&typetag)?;
        let zeros = b"\0\0\0\0";
        output.write_all(&zeros[..tag_pad])?;
        // Write the arguments
        Ok(output.write_all(&args)?)
    }
}

impl<'a> Serializer for &'a mut MsgSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ArgSerializer<'a>;
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
        Ok(ArgSerializer{ msg: self })
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
    // We can consider a unit, (), as a length-0 sequence
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_unit_struct(
        self,
        name: &'static str
    ) -> Result<Self::Ok, Self::Error>
    {
        Ok(())
    }

    default_ser!{bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char bytes
        str none some unit_variant newtype_struct newtype_variant
        tuple_struct tuple_variant map struct_variant}
}

impl<'a> Serializer for &'a mut ArgSerializer<'a> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<Self::Ok, Error>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeStruct = Self::SerializeSeq;
    type SerializeTupleStruct = Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Error>;
    type SerializeMap = Impossible<Self::Ok, Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Error>;

    fn serialize_i32(self, value: i32) -> ResultE<Self::Ok> {
        self.msg.addr_typetag.write_i32_tag()?;
        Ok(self.msg.args.osc_write_i32(value)?)
    }
    fn serialize_f32(self, value: f32) -> ResultE<Self::Ok> {
        self.msg.addr_typetag.write_f32_tag()?;
        Ok(self.msg.args.osc_write_f32(value)?)
    }
    fn serialize_str(self, value: &str) -> ResultE<Self::Ok> {
        self.msg.addr_typetag.write_str_tag()?;
        Ok(self.msg.args.osc_write_str(value)?)
    }
    fn serialize_bytes(self, value: &[u8]) -> ResultE<Self::Ok> {
        self.msg.addr_typetag.write_blob_tag()?;
        Ok(self.msg.args.osc_write_blob(value)?)
    }
    default_ser!{bool i8 i16 i64 u8 u16 u32 u64 f64 char
        none some unit unit_struct unit_variant newtype_struct newtype_variant
        seq tuple tuple_struct tuple_variant map struct struct_variant}
}

impl<'a> SerializeSeq for ArgSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<'b, T: ?Sized>(&'b mut self, value: &T) -> ResultE<()>
        where T: Serialize
    {
        // each element is an OSC arg: i32, f32, etc.
        value.serialize(&mut ArgSerializer{ msg: self.msg })
    }
    fn end(self) -> ResultE<()> {
        Ok(())
    }
}

impl<'a> SerializeStruct for ArgSerializer<'a> {
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

impl<'a> SerializeTuple for ArgSerializer<'a> {
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
