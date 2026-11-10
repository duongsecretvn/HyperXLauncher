//! High-performance zero-allocation VarInt and VarLong codec for Minecraft wire protocol.

use std::io::{self, Read, Write};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum VarIntError {
    #[error("VarInt is too wide (max 5 bytes)")]
    VarIntTooWide,
    #[error("VarLong is too wide (max 10 bytes)")]
    VarLongTooWide,
    #[error("Unexpected EOF while reading VarInt/VarLong")]
    UnexpectedEof,
    #[error("IO error: {0}")]
    Io(String),
}

impl From<io::Error> for VarIntError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::UnexpectedEof {
            VarIntError::UnexpectedEof
        } else {
            VarIntError::Io(err.to_string())
        }
    }
}

impl From<VarIntError> for io::Error {
    fn from(err: VarIntError) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, err.to_string())
    }
}

pub const MAX_VARINT_BYTES: usize = 5;
pub const MAX_VARLONG_BYTES: usize = 10;

/// Return byte length required to encode an `i32` as VarInt.
#[inline(always)]
pub fn varint_size(val: i32) -> usize {
    let mut v = val as u32;
    let mut bytes = 0;
    loop {
        v >>= 7;
        bytes += 1;
        if v == 0 {
            break;
        }
    }
    bytes
}

/// Encode `i32` as Minecraft VarInt into a byte slice. Returns number of bytes written.
#[inline]
pub fn encode_varint_slice(mut val: i32, out: &mut [u8]) -> Result<usize, VarIntError> {
    let mut i = 0;
    loop {
        if i >= out.len() {
            return Err(VarIntError::UnexpectedEof);
        }
        let mut byte = (val & 0x7F) as u8;
        val = ((val as u32) >> 7) as i32;
        if val != 0 {
            byte |= 0x80;
        }
        out[i] = byte;
        i += 1;
        if val == 0 {
            break;
        }
    }
    Ok(i)
}

/// Decode a VarInt from a slice. Returns (value, bytes_read).
#[inline]
pub fn decode_varint_slice(slice: &[u8]) -> Result<(i32, usize), VarIntError> {
    let mut result: i32 = 0;
    let mut shift = 0;
    let mut count = 0;

    for &byte in slice {
        if count >= MAX_VARINT_BYTES {
            return Err(VarIntError::VarIntTooWide);
        }
        result |= ((byte & 0x7F) as i32) << shift;
        count += 1;
        if (byte & 0x80) == 0 {
            return Ok((result, count));
        }
        shift += 7;
    }
    Err(VarIntError::UnexpectedEof)
}

/// Decode a VarLong from a slice. Returns (value, bytes_read).
#[inline]
pub fn decode_varlong_slice(slice: &[u8]) -> Result<(i64, usize), VarIntError> {
    let mut result: i64 = 0;
    let mut shift = 0;
    let mut count = 0;

    for &byte in slice {
        if count >= MAX_VARLONG_BYTES {
            return Err(VarIntError::VarLongTooWide);
        }
        result |= ((byte & 0x7F) as i64) << shift;
        count += 1;
        if (byte & 0x80) == 0 {
            return Ok((result, count));
        }
        shift += 7;
    }
    Err(VarIntError::UnexpectedEof)
}

/// Write VarInt to a writer.
#[inline]
pub fn write_varint<W: Write>(writer: &mut W, mut val: i32) -> io::Result<usize> {
    let mut buf = [0u8; MAX_VARINT_BYTES];
    let mut i = 0;
    loop {
        let mut byte = (val & 0x7F) as u8;
        val = ((val as u32) >> 7) as i32;
        if val != 0 {
            byte |= 0x80;
        }
        buf[i] = byte;
        i += 1;
        if val == 0 {
            break;
        }
    }
    writer.write_all(&buf[..i])?;
    Ok(i)
}

/// Read VarInt from a reader.
#[inline]
pub fn read_varint<R: Read>(reader: &mut R) -> Result<i32, VarIntError> {
    let mut result: i32 = 0;
    let mut shift = 0;
    let mut buf = [0u8; 1];

    for _ in 0..MAX_VARINT_BYTES {
        reader.read_exact(&mut buf)?;
        let byte = buf[0];
        result |= ((byte & 0x7F) as i32) << shift;
        if (byte & 0x80) == 0 {
            return Ok(result);
        }
        shift += 7;
    }
    Err(VarIntError::VarIntTooWide)
}

/// Write VarLong to a writer.
#[inline]
pub fn write_varlong<W: Write>(writer: &mut W, mut val: i64) -> io::Result<usize> {
    let mut buf = [0u8; MAX_VARLONG_BYTES];
    let mut i = 0;
    loop {
        let mut byte = (val & 0x7F) as u8;
        val = ((val as u64) >> 7) as i64;
        if val != 0 {
            byte |= 0x80;
        }
        buf[i] = byte;
        i += 1;
        if val == 0 {
            break;
        }
    }
    writer.write_all(&buf[..i])?;
    Ok(i)
}

/// Read VarLong from a reader.
#[inline]
pub fn read_varlong<R: Read>(reader: &mut R) -> Result<i64, VarIntError> {
    let mut result: i64 = 0;
    let mut shift = 0;
    let mut buf = [0u8; 1];

    for _ in 0..MAX_VARLONG_BYTES {
        reader.read_exact(&mut buf)?;
        let byte = buf[0];
        result |= ((byte & 0x7F) as i64) << shift;
        if (byte & 0x80) == 0 {
            return Ok(result);
        }
        shift += 7;
    }
    Err(VarIntError::VarLongTooWide)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_roundtrip() {
        let test_cases = [
            0,
            1,
            2,
            127,
            128,
            255,
            25565,
            2097151,
            2147483647,
            -1,
            -2147483648,
        ];

        for &val in &test_cases {
            let mut buf = Vec::new();
            let written = write_varint(&mut buf, val).expect("write failed");
            assert_eq!(varint_size(val), written);

            let (decoded, read) = decode_varint_slice(&buf).expect("decode failed");
            assert_eq!(val, decoded);
            assert_eq!(written, read);
        }
    }
}
