extern crate serde;
extern crate serde_osc;

use std::error::Error;
use std::fmt;
use std::io::Cursor;
use serde::Deserializer;
use serde::de::{SeqVisitor, Visitor};
use serde_osc::de::PktDeserializer;

#[test]
fn basic() {
    /// Struct we'll deserialize into
    #[derive(Debug, PartialEq, Eq)]
    struct Deserialized {
        address: String,
        arg_0: i32,
        arg_1: i32,
    }
    struct MyVisitor;
    impl Visitor for MyVisitor {
        type Value = Deserialized;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "An OSC path followed by two integer arguments")
        }
        fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where V: SeqVisitor
        {
            let address = visitor.visit()?.unwrap();
            let arg_0 = visitor.visit()?.unwrap();
            let arg_1 = visitor.visit()?.unwrap();
            Ok(Deserialized{
                address: address,
                arg_0: arg_0,
                arg_1: arg_1})
        }
    }
    let test_input = b"\x00\x00\x00\x1c/example/path\0\0\0,ii\0\x01\x02\x03\x04\x05\x06\x07\x08";
    let expected = Deserialized {
        address: "/example/path".to_owned(),
        arg_0: 0x01020304,
        arg_1: 0x05060708,
    };
    let rd = Cursor::new(test_input);
    let mut visitor = MyVisitor;
    let mut test_de = PktDeserializer::new(rd);
    let deserialized = test_de.deserialize(visitor).unwrap();
    assert_eq!(deserialized, expected);
}

