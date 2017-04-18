use std::convert::TryInto;
use serde::ser::{Impossible, Serialize, Serializer};

use super::error::{Error, ResultE};

pub struct TimetagSer {
    n_parsed: u8,
    parsed: [u32; 2],
}

impl TimetagSer {
    pub fn new() -> Self {
        TimetagSer {
            n_parsed: 0,
            parsed: [0, 0],
        }
    }
}

impl TryInto<(u32, u32)> for TimetagSer {
    type Error = Error;
    fn try_into(self) -> ResultE<(u32, u32)> {
        if self.n_parsed != 2 {
            return Err(Error::BadFormat);
        }
        Ok((self.parsed[0], self.parsed[1]))
    }
}

impl<'a> Serializer for &'a mut TimetagSer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<Self::Ok, Error>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeStruct = Self::SerializeSeq;
    type SerializeTupleStruct = Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Error>;
    type SerializeMap = Impossible<Self::Ok, Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Error>;

    fn serialize_u32(self, value: u32) -> ResultE<Self::Ok> {
        match self.parsed.get_mut(self.n_parsed as usize) {
            // Already serialized all the values needed for a timetag!
            None => Err(Error::BadFormat),
            Some(mut part) => {
                *part = value;
                self.n_parsed += 1;
                Ok(())
            }
        }
    }


    default_ser!{bool i8 i16 i32 i64 u8 u16 u64 f32 f64 char
        str bytes none some unit unit_struct unit_variant newtype_struct newtype_variant
        seq seq_fixed_size tuple tuple_struct tuple_variant map struct struct_variant}
}
