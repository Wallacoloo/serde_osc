#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_bytes;
extern crate serde_osc;

use serde_bytes::ByteBuf;
use serde_osc::{de, ser};

/// Struct we'll serialize.
/// This represents a single OSC message with three arguments:
///   one of type 'i', 'f' and 'b', encoded in the order they appear in the struct.
#[derive(Debug, Deserialize, Serialize)]
struct Message {
    address: String,
    // ByteBuf is the object we use for OSC "blobs".
    // It's a thin wrapper over Vec<u8> provided by Serde that allows
    // for more computationally-efficient serialization/deserialization.
    args: (i32, f32, ByteBuf),
}

fn main() {
    let message = Message {
        address: "/audio/play".to_owned(),
        args: (
            1,
            44100.0f32,
            ByteBuf::from(vec![0xde, 0xad, 0xbe, 0xef]),
        )
    };
    println!("Serializing {:?}", message);

    // Serialize the message to an OSC packet stored in a Vec<u8>
    let as_vec = ser::to_vec(&message).unwrap();
    println!("Serialied to: {:?}", as_vec);

    // Deserialize an OSC packet contained in a Vec<u8> into the Message struct
    let received: Message = de::from_slice(&as_vec).unwrap();
    println!("Received: {:?}", received);
}

