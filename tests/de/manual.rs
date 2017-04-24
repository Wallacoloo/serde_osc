use std::fmt;
use std::io::Cursor;
use serde::Deserializer as _Deserializer;
use serde_bytes::ByteBuf;
use serde::de::{SeqAccess, Visitor};
use serde_osc::de::Deserializer;

#[test]
fn manual_de() {
    /// Struct we'll deserialize into
    #[derive(Debug, PartialEq)]
    struct Deserialized {
        address: String,
        args: (i32, f32, ByteBuf),
    }
    struct MyVisitor;
    impl<'de> Visitor<'de> for MyVisitor {
        type Value = Deserialized;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "An OSC path followed by two integer arguments")
        }
        fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where V: SeqAccess<'de>
        {
            let address = visitor.next_element()?.unwrap();
            let args = visitor.next_element()?.unwrap();
            // Assert end of packet.
            assert!(visitor.next_element::<()>()? == None);
            Ok(Deserialized{ address, args })
        }
    }
    // Note: 0x43dc0000 is 440.0 in f32.
    let test_input = b"\x00\x00\x00\x2C/example/path\0\0\0,ifb\0\0\0\0\x01\x02\x03\x04\x43\xdc\0\0\0\0\0\x05\xde\xad\xbe\xef\xff\x00\x00\x00";
    let expected = Deserialized {
        address: "/example/path".to_owned(),
        args: (0x01020304,
            440.0,
            ByteBuf::from(vec![0xde, 0xad, 0xbe, 0xef, 0xff]),
        ),
    };
    let mut rd = Cursor::new(&test_input[..]);
    let visitor = MyVisitor;
    let mut test_de = Deserializer::new(&mut rd);
    let deserialized = test_de.deserialize_any(visitor).unwrap();
    assert_eq!(deserialized, expected);
}

