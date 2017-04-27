use std::io::Write;
use serde::ser::{Impossible, Serialize, Serializer, SerializeSeq, SerializeStruct, SerializeTuple};

use error::{Error, ResultE};
use super::bundle_serializer::BundleSerializer;
use super::msg_serializer::MsgSerializer;
use super::pkt_type_decoder::{PktType, PktTypeDecoder};

/// Serializes an entire OSC packet, which contains either one message or one
/// bundle.
///
/// The data being serialized may be either a tuple, a struct (in which case
/// each field is serialized in sequence, ignore the field names), or a Serde
/// sequence.
///
/// If the packet is a message, then the first field of the object being serialized
/// should be a `str`-type that represents the OSC address. The next field should
/// contain all the arguments. For example, the `message` instance below will serialize
/// to a message addressed to "/audio/play" with a payload of `4i32` and `0.5f32`.
///
/// ```
/// #[macro_use]
/// extern crate serde_derive;
///
/// #[derive(Serialize)]
/// struct AudioPlayer {
///     address: String,
///     args: (i32, f32),
/// }
///# fn main() {
/// let message = AudioPlayer{ address: "/audio/play".to_string(), args: (4, 0.5) };
///# }
/// ```
///
/// To serialize a bundle, simply omit the address field, add a `(u32, u32)` field
/// to transmit the [time-tag] associated with the bundle, and make sure all
/// subsequent fields are themselves something that is serializable as a message.
/// For example, an instance of the below `MyBundle` struct would serialize as a bundle.
///
/// ```
///# #[macro_use]
///# extern crate serde_derive;
///#
///# #[derive(Serialize)]
///# struct AudioPlayer {
///#     address: String,
///#     args: (i32, f32),
///# }
///#
/// #[derive(Serialize)]
/// struct MyBundle {
///     time: (u32, u32),
///     msg1: AudioPlayer,
///     msg2: AudioPlayer,
///     // ... And so on. Note that the messages don't need to be of homogeneous type;
///     // msg1 could be AudioPlayer and msg2 some other Serialize-able type.
///     // Additionally, this object would still be serialized as a bundle even
///     // if it contained only one message.
/// }
///# fn main() {}
/// ```
///
/// Note: the time-tag can also be `[u32; 2]`, a struct containing two `u32` members,
/// or *anything* that serializes as a flat sequence of two `u32`s.
///
/// [time-tag]: http://opensoundcontrol.org/node/3/#timetags
pub struct PktSerializer<W: Write> {
    output: W,
}

/// After the State receives a serialize_seq call,
/// we return one of these. If the next data is a string, then it's the message
/// address & we're serializing a message.
/// If the next data is a sequence, we're serializing a bundle, and that sequence is
/// the (u32, u32) time tag.
pub struct PktContents<'a, W: Write + 'a> {
    output: &'a mut PktSerializer<W>,
    state: State,
}

enum State {
    UnknownType,
    Msg(MsgSerializer),
    Bundle(BundleSerializer),
}


impl<W: Write> PktSerializer<W> {
    pub fn new(output: W) -> Self {
        Self{ output }
    }
}

impl<'a, W: Write> Serializer for &'a mut PktSerializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = PktContents<'a, W>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeStruct = Self::SerializeSeq;
    type SerializeTupleStruct = Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Error>;
    type SerializeMap = Impossible<Self::Ok, Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Error>;

    default_ser!{bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char
        str bytes none some unit unit_struct unit_variant newtype_struct newtype_variant
        tuple_struct tuple_variant map struct_variant}
    fn serialize_seq(
        self, 
        _size: Option<usize>
    ) -> ResultE<Self::SerializeSeq>
    {
        Ok(PktContents{ output: self, state: State::UnknownType })
    }
    fn serialize_tuple(
        self, 
        size: usize
    ) -> ResultE<Self::SerializeTuple>
    {
        self.serialize_seq(Some(size))
    }
    fn serialize_struct(
        self, 
        _: &'static str, 
        size: usize
    ) -> ResultE<Self::SerializeStruct>
    {
        self.serialize_seq(Some(size))
    }
}


impl<'a, W: Write + 'a> SerializeSeq for PktContents<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<'b, T: ?Sized>(&'b mut self, value: &T) -> ResultE<()>
        where T: Serialize
    {
        match self.state {
            State::UnknownType => {
                // Determine the type of the packet.
                // If the first element is a string, then we become a message;
                //   will accept i32, f32, blob, str args.
                // If the first element we see is a timecode (seq of u32, u32),
                //   then we become a bundle.
                let mut decoder = PktTypeDecoder::new();
                value.serialize(&mut decoder)?;

                match decoder.pkt_type() {
                    PktType::Unknown => Err(Error::BadFormat),
                    PktType::Msg => {
                        self.state = State::Msg(MsgSerializer::new(
                            decoder.data()
                        )?);
                        Ok(())
                    },
                    PktType::Bundle => {
                        self.state = State::Bundle(BundleSerializer::new(
                            decoder.data()
                        ));
                        Ok(())
                    },
                }
            },
            State::Msg(ref mut msg) => {
                value.serialize(msg)
            },
            State::Bundle(ref mut bundle) => {
                value.serialize(bundle)
            },
        }
    }

    fn end(self) -> ResultE<()> {
        match self.state {
            // Packet has no contents!
            State::UnknownType => Err(Error::BadFormat),
            // Write the message header & data to the output
            State::Msg(msg) => {
                msg.write_into(&mut self.output.output)
            },
            // Write the bundle header & data to the output
            State::Bundle(bundle) => {
                bundle.write_into(&mut self.output.output)
            }
        }
    }
}

impl<'a, W: Write + 'a> SerializeStruct for PktContents<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> ResultE<()>
        where T: Serialize
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> ResultE<()> {
        SerializeSeq::end(self)
    }
}

impl<'a, W: Write + 'a> SerializeTuple for PktContents<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> ResultE<()>
        where T: Serialize
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> ResultE<()> {
        SerializeSeq::end(self)
    }
}

