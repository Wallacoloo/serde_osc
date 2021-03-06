use std::io::{Cursor, Write};
use serde::Serialize;
use serde_bytes::ByteBuf;
use serde_osc::ser::Serializer;


#[test]
fn auto_ser() {
    /// Struct we'll serialize
    #[derive(Debug, PartialEq, Serialize)]
    struct Serialized {
        address: String,
        args: (i32, f32, ByteBuf),
    }
    let test_input = Serialized {
        address: "/example/path".to_owned(),
        args: (
            0x01020304,
            440.0,
            ByteBuf::from(vec![0xde, 0xad, 0xbe, 0xef, 0xff]),
        ),
    };

    // Note: 0x43dc0000 is 440.0 in f32.
    let expected = b"\x00\x00\x00\x2C/example/path\0\0\0,ifb\0\0\0\0\x01\x02\x03\x04\x43\xdc\0\0\0\0\0\x05\xde\xad\xbe\xef\xff\x00\x00\x00".to_vec();
    let mut output = Cursor::new(Vec::new());

    {
        let mut test_de = Serializer::new(output.by_ref());
        let _result = test_input.serialize(&mut test_de).unwrap();
    }
    assert_eq!(output.into_inner(), expected);
}
