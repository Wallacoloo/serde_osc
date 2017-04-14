use std::fmt;
use std::io::Cursor;
use serde::Deserializer;
use serde::bytes::ByteBuf;
use serde::de::{SeqVisitor, Visitor};
use serde_osc::de::PktDeserializer;

#[test]
fn manual_de() {
    /// Struct we'll deserialize into
    #[derive(Debug, PartialEq)]
    struct Deserialized {
        address: String,
        arg_0: i32,
        arg_1: f32,
        arg_2: ByteBuf,
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
            let arg_2 = visitor.visit()?.unwrap();
            // Assert end of packet.
            assert!(visitor.visit::<()>()? == None);
            Ok(Deserialized{ address, arg_0, arg_1, arg_2 })
        }
    }
    // Note: 0x43dc0000 is 440.0 in f32.
    let test_input = b"\x00\x00\x00\x2C/example/path\0\0\0,ifb\0\0\0\0\x01\x02\x03\x04\x43\xdc\0\0\0\0\0\x05\xde\xad\xbe\xef\xff\x00\x00\x00";
    let expected = Deserialized {
        address: "/example/path".to_owned(),
        arg_0: 0x01020304,
        arg_1: 440.0,
        arg_2: ByteBuf::from(vec![0xde, 0xad, 0xbe, 0xef, 0xff]),
    };
    let rd = Cursor::new(&test_input[..]);
    let visitor = MyVisitor;
    let mut test_de = PktDeserializer::new(rd);
    let deserialized = test_de.deserialize(visitor).unwrap();
    assert_eq!(deserialized, expected);
}

