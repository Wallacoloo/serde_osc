use serde::ser::{Impossible, Serialize, Serializer, SerializeSeq};

use super::error::{Error, ResultE};

pub struct PktSerializer {
    /// Because OSC makes use of length prefixes,
    /// we have to buffer the entire output before we can write the length.
    /// TODO: we can use a seekable writer to mitigate this.
    payload: Vec<u8>,
}

impl<'a> Serializer for &'a mut PktSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Impossible<Self::Ok, Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Error>;
    type SerializeMap = Impossible<Self::Ok, Error>;
    type SerializeStruct = Impossible<Self::Ok, Error>;
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
    fn serialize_i32(self, _: i32) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
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
    fn serialize_f32(self, _: f32) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_f64(self, _: f64) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_char(self, _: char) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_str(self, _: &str) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
    }
    fn serialize_bytes(self, _: &[u8]) -> ResultE<Self::Ok> {
        Err(Error::UnsupportedType)
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
       Ok(Compound{ osc_ser: self })
    }
    fn serialize_seq_fixed_size(
        self, 
        _: usize
    ) -> ResultE<Self::SerializeSeq>
    {
        Err(Error::UnsupportedType)
    }
    fn serialize_tuple(
        self, 
        _: usize
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
        _: usize
    ) -> ResultE<Self::SerializeStruct>
    {
        Err(Error::UnsupportedType)
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


pub struct Compound<'a> {
    osc_ser: &'a mut PktSerializer,
}

impl<'a> SerializeSeq for Compound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, _: &T) -> ResultE<()>
        where T: Serialize
    {
        unimplemented!()
    }

    fn end(self) -> ResultE<()> {
        unimplemented!()
    }
}
