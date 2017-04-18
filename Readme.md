Serialization and deserialization of Open Sound Control 1.0 packets represented using structs/tuples/anything supported by serde.

Supports the 4 types specified in OSC 1.0: 'f', 'i', 's', 'b' corresponding to `f32`, `i32`, `String` and `Vec<u8>` ("blobs"), respectively.
Also supports nesting of OSC bundles.

For usage, refer to the `tests` directory inside `serde_osc`.
Examples will be published once the library is more stable/fully developed.

Development status
------------------
 - [x] Deserialization of both OSC messages and bundles into any sequence type (structs, tuples, etc).
 - [x] Serialization of both OSC messages and bundles from any sequence type (structs, tuples, etc).
 - [ ] Macros to aid in addressing messages (e.g. serializing the address of a
message without needing to store that address as a String in the struct/tuple being serialized)
 - [ ] Macros to aid in translating OSC's time format to std::time

This library is under active development and will likely see some interface-breaking changes.


License
-------
Serde OSC follows the same licensing as Serde. It is licensed under either of

   * Apache License, Version 2.0, (http://www.apache.org/licenses/LICENSE-2.0)
   * MIT license (http://opensource.org/licenses/MIT)

at your option.
