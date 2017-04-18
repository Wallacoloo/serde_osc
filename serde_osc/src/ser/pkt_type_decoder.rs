use std::io::Cursor;
use serde::ser::{Impossible, Serialize, Serializer};

use super::error::{Error, ResultE};
use super::osc_writer::OscWriter;

pub struct PktTypeDecoder {
    output: Cursor<Vec<u8>>,
    pkt_type: PktType,
}

#[derive(Copy, Clone)]
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
    type SerializeSeq = Impossible<Self::Ok, Error>;
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

    default_ser!{bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char
        bytes none some unit unit_struct unit_variant newtype_struct newtype_variant
        seq seq_fixed_size tuple tuple_struct tuple_variant map struct struct_variant}

}
