use std::convert::TryInto;
use std::io::{Cursor, Write};
use byteorder::WriteBytesExt;
use serde::ser::{Impossible, Serialize, Serializer, SerializeSeq, SerializeStruct, SerializeTuple};

use error::{Error, ResultE};
use super::osc_writer::OscWriter;
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

/// Once we know we're serializing a message, we do so through this struct.
struct MsgSerializer {
    /// Address + typetag, merged into one field
    addr_typetag: Cursor<Vec<u8>>,
    /// Binary-formatted argument data
    args: Cursor<Vec<u8>>,
}

struct ArgSerializer<'a> {
    msg: &'a mut MsgSerializer,
}

struct BundleSerializer {
    contents: Cursor<Vec<u8>>,
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


impl<'a> Serializer for &'a mut MsgSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ArgSerializer<'a>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeStruct = Self::SerializeSeq;
    type SerializeTupleStruct = Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Error>;
    type SerializeMap = Impossible<Self::Ok, Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Error>;

    fn serialize_seq(
        self, 
        _size: Option<usize>
    ) -> ResultE<Self::SerializeSeq>
    {
        Ok(ArgSerializer{ msg: self })
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

    default_ser!{bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char bytes
        str none some unit unit_struct unit_variant newtype_struct newtype_variant
        tuple_struct tuple_variant map struct_variant}
}

impl<'a> Serializer for &'a mut ArgSerializer<'a> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<Self::Ok, Error>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeStruct = Self::SerializeSeq;
    type SerializeTupleStruct = Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Error>;
    type SerializeMap = Impossible<Self::Ok, Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Error>;

    fn serialize_i32(self, value: i32) -> ResultE<Self::Ok> {
        self.msg.addr_typetag.write_i32_tag()?;
        Ok(self.msg.args.osc_write_i32(value)?)
    }
    fn serialize_f32(self, value: f32) -> ResultE<Self::Ok> {
        self.msg.addr_typetag.write_f32_tag()?;
        Ok(self.msg.args.osc_write_f32(value)?)
    }
    fn serialize_str(self, value: &str) -> ResultE<Self::Ok> {
        self.msg.addr_typetag.write_str_tag()?;
        Ok(self.msg.args.osc_write_str(value)?)
    }
    fn serialize_bytes(self, value: &[u8]) -> ResultE<Self::Ok> {
        self.msg.addr_typetag.write_blob_tag()?;
        Ok(self.msg.args.osc_write_blob(value)?)
    }
    default_ser!{bool i8 i16 i64 u8 u16 u32 u64 f64 char
        none some unit unit_struct unit_variant newtype_struct newtype_variant
        seq tuple tuple_struct tuple_variant map struct struct_variant}
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
                        let mut addr_typetag = decoder.data();
                        // Prepare to append type arguments in future calls
                        addr_typetag.write_u8(b',')?;
                        self.state = State::Msg(MsgSerializer{
                            addr_typetag,
                            args: Cursor::new(Vec::new()),
                        });
                        Ok(())
                    },
                    PktType::Bundle => {
                        self.state = State::Bundle(BundleSerializer{
                            contents: decoder.data(),
                        });
                        Ok(())
                    },
                }
            },
            State::Msg(ref mut msg) => {
                value.serialize(msg)
            },
            State::Bundle(ref mut bundle) => {
                // each bundle element is itself a packet.
                let mut ser = PktSerializer{ output: bundle.contents.by_ref() };
                value.serialize(&mut ser)
            },
        }
    }

    fn end(self) -> ResultE<()> {
        match self.state {
            // Packet has no contents!
            State::UnknownType => Err(Error::BadFormat),
            // Write the message header & data to the output
            State::Msg(msg) => {
                let typetag = msg.addr_typetag.into_inner();
                let args = msg.args.into_inner();
                let tag_pad = 4 - (typetag.len() % 4);
                let payload_size = typetag.len() + tag_pad + args.len();
                if payload_size % 4 != 0 {
                    // Sanity check; OSC requires packets to be a multiple of 4 bytes.
                    return Err(Error::BadFormat);
                }

                // Write the packet length
                self.output.output.osc_write_i32(payload_size.try_into()?)?;
                // Write the address and type tag
                self.output.output.write_all(&typetag)?;
                let zeros = b"\0\0\0\0";
                self.output.output.write_all(&zeros[..tag_pad])?;
                // Write the arguments
                Ok(self.output.output.write_all(&args)?)
            },
            // Write the bundle header & data to the output
            State::Bundle(bundle) => {
                let payload = bundle.contents.into_inner();
                // Add 8 because we have yet to write the #bundle address
                let payload_size = 8 + payload.len();
                if payload_size % 4 != 0 {
                    // Sanity check; OSC requires packets to be a multiple of 4 bytes.
                    return Err(Error::BadFormat);
                }
                // Write the packet length
                let output = &mut self.output.output;
                output.osc_write_i32(payload_size.try_into()?)?;
                // Write the packet payload
                output.osc_write_str("#bundle")?;
                Ok(output.write_all(&payload)?)
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

impl<'a> SerializeSeq for ArgSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<'b, T: ?Sized>(&'b mut self, value: &T) -> ResultE<()>
        where T: Serialize
    {
        // each element is an OSC arg: i32, f32, etc.
        value.serialize(&mut ArgSerializer{ msg: self.msg })
    }
    fn end(self) -> ResultE<()> {
        Ok(())
    }
}

impl<'a> SerializeStruct for ArgSerializer<'a> {
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

impl<'a> SerializeTuple for ArgSerializer<'a> {
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
