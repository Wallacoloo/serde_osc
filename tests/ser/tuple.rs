use std::io::{Cursor, Write};
use serde::Serialize;
use serde_bytes::ByteBuf;
use serde_osc::ser::Serializer;


#[test]
fn tuple_ser() {
    /// Tuple we'll serialize
    let test_input = (
        "/example/path".to_owned(),
        (
            0x01020304i32,
            440.0f32,
            ByteBuf::from(vec![0xde, 0xad, 0xbe, 0xef, 0xff]),
        )
    );

    // Note: 0x43dc0000 is 440.0 in f32.
    let expected = b"/example/path\0\0\0,ifb\0\0\0\0\x01\x02\x03\x04\x43\xdc\0\0\0\0\0\x05\xde\xad\xbe\xef\xff\x00\x00\x00".to_vec();
    let mut output = Cursor::new(Vec::new());

    {
        let mut test_de = Serializer::new(output.by_ref());
        let _result = test_input.serialize(&mut test_de).unwrap();
    }
    assert_eq!(output.into_inner(), expected);
}

#[test]
fn unit_ser() {
    /// Tuple we'll serialize
    let test_input = (
        "/ts".to_owned(), ()
    );

    let expected = b"/ts\0,\0\0\0".to_vec();
    let mut output = Cursor::new(Vec::new());

    {
        let mut test_de = Serializer::new(output.by_ref());
        let _result = test_input.serialize(&mut test_de).unwrap();
    }
    assert_eq!(output.into_inner(), expected);
}
