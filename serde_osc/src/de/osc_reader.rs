use std::convert::TryInto;
use std::io::Read;
use byteorder::{BigEndian, ReadBytesExt};

use super::error::{Error, ResultE};

/// auto-implemented trait to parse OSC data from a Read object.
pub trait OscReader: Read {
    /// Read a null-terminated sequence of bytes & verify padding
    fn read_0term_bytes(&mut self) -> ResultE<Vec<u8>> {
        let mut data = Vec::new();
        // Because of the 4-byte required padding, we can process 4 characters at a time
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        let mut num_zeros = 0;
        while num_zeros == 0 {
            self.read_exact(&mut buf)?;
            // Copy the NON-NULL characters to the buffer.
            num_zeros = buf.iter().filter(|c| **c == 0).count();
            if buf[4-num_zeros..4].iter().any(|c| *c != 0) {
                // We had data after the null terminator.
                return Err(Error::BadPadding);
            }
            data.extend_from_slice(&buf[0..4-num_zeros]);
        }
        Ok(data)
    }
    /// Read a null-terminated UTF-8 string & verify padding
    fn parse_str(&mut self) -> ResultE<String> {
        // Note: although OSC specifies ascii only, we may have data >= 128 in the vector.
        // We can safely assume a UTF-8 encoding, because no byte of any multibyte UTF-8
        // contains a zero; the only zero possible in a UTF-8 string is the ASCII zero.
        // See the UTF-8 table here: https://en.wikipedia.org/wiki/UTF-8#History
        let bytes = self.read_0term_bytes()?;
        Ok(String::from_utf8(bytes)?)
    }
    fn parse_i32(&mut self) -> ResultE<i32> {
       Ok( self.read_i32::<BigEndian>()?)
    }
    fn parse_f32(&mut self) -> ResultE<f32> {
        Ok(self.read_f32::<BigEndian>()?)
    }
    /// "Time tags are represented by a 64 bit fixed point number.
    ///  The first 32 bits specify the number of seconds since midnight on January 1, 1900,
    ///  and the last 32 bits specify fractional parts of a second to a precision of about 200 picoseconds.
    ///  This is the representation used by Internet NTP timestamps."
    fn parse_timetag(&mut self) -> ResultE<(u32, u32)> {
       let sec = self.read_u32::<BigEndian>()?;
       let frac = self.read_u32::<BigEndian>()?;
       Ok((sec, frac))
    }
    /// Read an OSC blob & verify padding.
    /// A blob consists of a length + u8 array.
    fn parse_blob(&mut self) -> ResultE<Vec<u8>> {
        let size: usize = self.parse_i32()?.try_into()?;
        // Blobs are padded to a 4-byte boundary
        let padded_size = (size + 3) & !0x3;
        // Read EXACTLY this much data:
        let mut data = vec![0; padded_size];
        self.read_exact(&mut data)?;
        // Ensure these extra bytes where NULL (sanity check)
        if data.drain(size..padded_size).all(|c| c == 0) {
            Ok(data)
        } else {
            Err(Error::BadPadding)
        }
    }
}

/// Provide OSC reading functions to all types implementing Read
impl<R: Read + ?Sized> OscReader for R {}

