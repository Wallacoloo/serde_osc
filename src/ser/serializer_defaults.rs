//! Provide default implementations of serialize_xxx
//! to implement the serde::ser::Serializer trait.
#[doc(hidden)]
#[macro_export]
macro_rules! default_ser_one {
    ($result:ident, $func:ident($($arg:ty),*)) => {
        fn $func(self, $(_: $arg,)*) -> ResultE<Self::$result> {
            Err(Error::UnsupportedType)
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! default_ser_one_sized {
    ($func:ident($($arg:ty),*)) => {
        fn $func<T: ?Sized + Serialize>(self, $(_: $arg,)*) -> ResultE<Self::Ok> {
            Err(Error::UnsupportedType)
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! default_ser_helper {
    (bool) => { default_ser_one!{Ok, serialize_bool(bool)} };
    (i8) => { default_ser_one!{Ok, serialize_i8(i8)} };
    (i16) => { default_ser_one!{Ok, serialize_i16(i16)} };
    (i32) => { default_ser_one!{Ok, serialize_i32(i32)} };
    (i64) => { default_ser_one!{Ok, serialize_i64(i64)} };
    (u8) => { default_ser_one!{Ok, serialize_u8(u8)} };
    (u16) => { default_ser_one!{Ok, serialize_u16(u16)} };
    (u32) => { default_ser_one!{Ok, serialize_u32(u32)} };
    (u64) => { default_ser_one!{Ok, serialize_u64(u64)} };
    (f32) => { default_ser_one!{Ok, serialize_f32(f32)} };
    (f64) => { default_ser_one!{Ok, serialize_f64(f64)} };
    (char) => { default_ser_one!{Ok, serialize_char(char)} };
    (str) => { default_ser_one!{Ok, serialize_str(&str)} };
    (bytes) => { default_ser_one!{Ok, serialize_bytes(&[u8])} };
    (none) => { default_ser_one!{Ok, serialize_none()} };
    (some) => { default_ser_one_sized!{serialize_some(&T)} };
    (unit) => { default_ser_one!{Ok, serialize_unit()} };
    (unit_struct) => { default_ser_one!{Ok, serialize_unit_struct(&'static str)} };
    (unit_variant) => { default_ser_one!{Ok, serialize_unit_variant(&'static str, usize, &'static str)} };
    (newtype_struct) => { default_ser_one_sized!{serialize_newtype_struct(&'static str, &T)} };
    (newtype_variant) => { default_ser_one_sized!{serialize_newtype_variant(&'static str, usize, &'static str, &T)} };
    (seq) => { default_ser_one!{SerializeSeq, serialize_seq(Option<usize>)} };
    (seq_fixed_size) => { default_ser_one!{SerializeSeq, serialize_seq_fixed_size(usize)} };
    (tuple) => { default_ser_one!{SerializeTuple, serialize_tuple(usize)} };
    (tuple_struct) => { default_ser_one!{SerializeTupleStruct, serialize_tuple_struct(&'static str, usize)} };
    (tuple_variant) => { default_ser_one!{SerializeTupleVariant, serialize_tuple_variant(&'static str, usize, &'static str, usize)} };
    (map) => { default_ser_one!{SerializeMap, serialize_map(Option<usize>)} };
    (struct) => { default_ser_one!{SerializeStruct, serialize_struct(&'static str, usize)} };
    (struct_variant) => { default_ser_one!{SerializeStructVariant, serialize_struct_variant(&'static str, usize, &'static str, usize)} };
}

#[doc(hidden)]
#[macro_export]
macro_rules! default_ser {
    ($($func:ident)*) => {
        $(default_ser_helper!{$func})*
    };
}

