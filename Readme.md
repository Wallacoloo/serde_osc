Deserialization of Open Sound Control 1.0 packets into structs/tuples/anything deserializable through serde.

Supports the 4 types specified in OSC 1.0: 'f', 'i', 's', 'b' corresponding to `f32`, `i32`, `String` and `Vec<u8>` (blob), respectively.
Also supports nested bundles.

Currently, only deserialization is implemented. Serialization will come soon. Routing/address matching will most likely not be implemented in this crate.

For usage, refer to the `tests` directory inside `serde_osc`.
Examples will be published once the library is more stable/fully developed.
