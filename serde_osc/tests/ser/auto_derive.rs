use std::io::Cursor;
use serde::Serialize;
use serde::bytes::ByteBuf;
use serde_osc::ser::PktSerializer;


#[test]
fn auto_de() {
    /// Struct we'll serialize
    #[derive(Debug, PartialEq, Serialize)]
    struct Serialized {
        address: String,
        arg_0: i32,
        arg_1: f32,
        arg_2: ByteBuf,
    }
    let test_input = Serialized {
        address: "/example/path".to_owned(),
        arg_0: 0x01020304,
        arg_1: 440.0,
        arg_2: ByteBuf::from(vec![0xde, 0xad, 0xbe, 0xef, 0xff]),
    };

    // Note: 0x43dc0000 is 440.0 in f32.
    let expected = b"\x00\x00\x00\x2C/example/path\0\0\0,ifb\0\0\0\0\x01\x02\x03\x04\x43\xdc\0\0\0\0\0\x05\xde\xad\xbe\xef\xff\x00\x00\x00";

    let mut test_de = PktSerializer::new();
    let serialized = test_input.serialize(&mut test_de).unwrap();
    //assert_eq!(deserialized, expected);
}
