use std::convert::TryInto;
use std::io::Write;
use byteorder::{BigEndian, WriteBytesExt};

use super::error::ResultE;

/// auto-implemented trait to write OSC data to a Write object.
pub trait OscWriter: Write {
    fn osc_write_i32(&mut self, value: i32) -> ResultE<()> {
        Ok(self.write_i32::<BigEndian>(value)?)
    }
    fn write_i32_tag(&mut self) -> ResultE<()> {
        Ok(self.write_u8(b'i')?)
    }
    fn osc_write_f32(&mut self, value: f32) -> ResultE<()> {
        Ok(self.write_f32::<BigEndian>(value)?)
    }
    fn write_f32_tag(&mut self) -> ResultE<()> {
        Ok(self.write_u8(b'f')?)
    }
    fn osc_write_str(&mut self, value: &str) -> ResultE<()> {
        self.write_all(value.as_bytes())?;
        // pad to 4-byte boundary, PLUS ensure we have at least one null terminator.
        let pad_bytes = 4 - (value.len() % 4);
        let zeros = b"\0\0\0\0";
        Ok(self.write_all(&zeros[..pad_bytes])?)
    }
    fn write_str_tag(&mut self) -> ResultE<()> {
        Ok(self.write_u8(b's')?)
    }
    fn osc_write_blob(&mut self, value: &[u8]) -> ResultE<()> {
        // write the blob length (yes, as an i32)
        self.write_i32::<BigEndian>(value.len().try_into()?)?;
        self.write_all(value)?;
        let pad_bytes = (4 - value.len() % 4) % 4;
        let zeros = b"\0\0\0\0";
        Ok(self.write_all(&zeros[..pad_bytes])?)
    }
    fn write_blob_tag(&mut self) -> ResultE<()> {
        Ok(self.write_u8(b'b')?)
    }
    /// Write the OSC timetag, characterized by a (u32, u32) pair.
    /// The first u32 is the seconds, second is fraction of seconds.
    fn osc_write_timetag(&mut self, tag: (u32, u32)) -> ResultE<()> {
        self.write_u32::<BigEndian>(tag.0)?;
        self.write_u32::<BigEndian>(tag.1)?;
        Ok(())
    }
}

/// Provide OSC writing functions to all types implementing Write
impl<W: Write + ?Sized> OscWriter for W {}

