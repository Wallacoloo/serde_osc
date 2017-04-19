#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_osc;

use serde::bytes::ByteBuf;
use serde_osc::ser;

/// Struct we'll serialize.
/// This represents a single OSC message with three arguments:
///   one of type 'i', 'f' and 'b', sent in the order they appear in the struct.
#[derive(Debug, PartialEq, Serialize)]
struct Serialized {
    address: String,
    num_channels: i32,
    rate: f32,
    content: ByteBuf,
}

fn main() {
    let message = Serialized {
        address: "/audio/play".to_owned(),
        num_channels: 1,
        rate: 44100.0f32,
        content: ByteBuf::from(vec![0xde, 0xad, 0xbe, 0xef])
    };
    let as_vec = ser::to_vec(&message).unwrap();
    println!("Serialied to: {:?}", as_vec);
}
