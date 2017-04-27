use serde_bytes::ByteBuf;
use serde_osc::de;


#[test]
fn auto_de() {
    /// Struct we'll deserialize into
    #[derive(Debug, PartialEq, Deserialize)]
    struct Deserialized {
        address: String,
        args: (i32, f32, ByteBuf),
    }
    let expected = Deserialized {
        address: "/example/path".to_owned(),
        args: (
            0x01020304,
            440.0,
            ByteBuf::from(vec![0xde, 0xad, 0xbe, 0xef, 0xff]),
        ),
    };

    // Note: 0x43dc0000 is 440.0 in f32.
    let test_input = b"\x00\x00\x00\x2C/example/path\0\0\0,ifb\0\0\0\0\x01\x02\x03\x04\x43\xdc\0\0\0\0\0\x05\xde\xad\xbe\xef\xff\x00\x00\x00";

    let deserialized: Deserialized = de::from_slice(test_input).unwrap();
    assert_eq!(deserialized, expected);
}

#[test]
fn to_tuple() {
    let expected = ("/ts".to_owned(), ());
    let test_input = b"\x00\x00\x00\x08/ts\0,\0\0\0";
    let deserialized: (String, ()) = de::from_slice(test_input).unwrap();
    assert_eq!(deserialized, expected);
}

#[test]
fn to_array() {
    let expected: (String, [i32; 0]) = ("/ts".to_owned(), []);
    let test_input = b"\x00\x00\x00\x08/ts\0,\0\0\0";
    let deserialized: (String, [i32; 0]) = de::from_slice(test_input).unwrap();
    assert_eq!(deserialized, expected);
}

#[test]
fn to_unit_struct() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Unit;

    let expected: (String, Unit) = ("/ts".to_owned(), Unit);
    let test_input = b"\x00\x00\x00\x08/ts\0,\0\0\0";
    let deserialized: (String, Unit) = de::from_slice(test_input).unwrap();
    assert_eq!(deserialized, expected);
}

