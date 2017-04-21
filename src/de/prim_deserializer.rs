use serde::de::{Deserializer, Visitor};

use error::{Error, ResultE};

/// Serde gymnastics.
/// Implements the Deserializer trait for primitive types.
/// Currently only implemented for the types needed by osc_serde,
/// but could easily be extended via a macro.
pub struct PrimDeserializer<T>(pub T);

impl<'de> Deserializer<'de> for PrimDeserializer<u32> {
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> ResultE<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_u32(self.0)
    }

    // Ignore type hints
    // More info: https://github.com/serde-rs/serde/blob/b7d6c5d9f7b3085a4d40a446eeb95976d2337e07/serde/src/macros.rs#L106
    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct identifier tuple enum ignored_any
    }
}
