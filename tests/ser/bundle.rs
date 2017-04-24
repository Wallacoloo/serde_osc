use std::io::{Cursor, Write};
use serde::Serialize;
use serde_osc::ser::Serializer;

#[test]
fn bundle() {
    #[derive(Debug, PartialEq, Serialize)]
    struct Msg1 {
        address: String,
        args: (i32,),
    }
    #[derive(Debug, PartialEq, Serialize)]
    struct Msg2 {
        address: String,
        args: (f32,),
    }
    #[derive(Debug, PartialEq, Serialize)]
    struct Bundle {
        timestamp: (u32, u32),
        messages: (Msg1, Msg2),
    }
    let test_input = Bundle {
        timestamp: (0x01020304, 0x05060708),
        messages: (
            Msg1 {
                address: "/m1".to_owned(),
                args: (0x5eeeeeed,),
            },
            Msg2 {
                address: "/m2".to_owned(),
                args: (440.0,),
            }
        )
    };

    // Note: 0x43dc0000 is 440.0 in f32.
    let expected = b"\x00\x00\x00\x30#bundle\0\x01\x02\x03\x04\x05\x06\x07\x08\x00\x00\x00\x0C/m1\0,i\0\0\x5E\xEE\xEE\xED\x00\x00\x00\x0C/m2\0,f\0\0\x43\xdc\x00\x00".to_vec();
    let mut output = Cursor::new(Vec::new());

    {
        let mut test_de = Serializer::new(output.by_ref());
        let _result = test_input.serialize(&mut test_de).unwrap();
    }
    assert_eq!(output.into_inner(), expected);
}
