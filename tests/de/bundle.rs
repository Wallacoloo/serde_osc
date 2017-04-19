use std::io::Cursor;
use serde::Deserialize;
use serde_osc::de::Deserializer;

#[test]
fn bundle() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Msg1 {
        address: String,
        arg_0: i32,
    }
    #[derive(Debug, PartialEq, Deserialize)]
    struct Msg2 {
        address: String,
        arg_0: f32,
    }
    #[derive(Debug, PartialEq, Deserialize)]
    struct Bundle {
        timestamp: (u32, u32),
        msg1: Msg1,
        msg2: Msg2,
    }
    let expected = Bundle {
        timestamp: (0x01020304, 0x05060708),
        msg1: Msg1 {
            address: "/m1".to_owned(),
            arg_0: 0x5eeeeeed,
        },
        msg2: Msg2 {
            address: "/m2".to_owned(),
            arg_0: 440.0,
        }
    };

    // Note: 0x43dc0000 is 440.0 in f32.
    let test_input = b"\x00\x00\x00\x30#bundle\0\x01\x02\x03\x04\x05\x06\x07\x08\x00\x00\x00\x0C/m1\0,i\0\0\x5E\xEE\xEE\xED\x00\x00\x00\x0C/m2\0,f\0\0\x43\xdc\x00\x00";

    let rd = Cursor::new(&test_input[..]);
    let mut test_de = Deserializer::new(rd);
    let deserialized = Bundle::deserialize(&mut test_de).unwrap();
    assert_eq!(deserialized, expected);
}
