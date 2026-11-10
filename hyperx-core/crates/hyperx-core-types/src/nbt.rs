//! Zero-copy high-performance NBT (Named Binary Tag) engine for Minecraft.

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::BTreeMap;
use std::io::{self, Cursor, Read, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NbtError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Unknown NBT tag ID: {0}")]
    InvalidTagId(u8),
    #[error("NBT parse error: {0}")]
    ParseError(String),
}

pub type Compound = BTreeMap<String, NbtTag>;

#[derive(Clone, Debug, PartialEq)]
pub enum NbtTag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<NbtTag>),
    Compound(Compound),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NbtTag {
    pub fn id(&self) -> u8 {
        match self {
            NbtTag::End => 0,
            NbtTag::Byte(_) => 1,
            NbtTag::Short(_) => 2,
            NbtTag::Int(_) => 3,
            NbtTag::Long(_) => 4,
            NbtTag::Float(_) => 5,
            NbtTag::Double(_) => 6,
            NbtTag::ByteArray(_) => 7,
            NbtTag::String(_) => 8,
            NbtTag::List(_) => 9,
            NbtTag::Compound(_) => 10,
            NbtTag::IntArray(_) => 11,
            NbtTag::LongArray(_) => 12,
        }
    }

    pub fn as_compound(&self) -> Option<&Compound> {
        if let NbtTag::Compound(ref c) = self {
            Some(c)
        } else {
            None
        }
    }

    pub fn as_compound_mut(&mut self) -> Option<&mut Compound> {
        if let NbtTag::Compound(ref mut c) = self {
            Some(c)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let NbtTag::String(ref s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            NbtTag::Byte(b) => Some(*b as i32),
            NbtTag::Short(s) => Some(*s as i32),
            NbtTag::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            NbtTag::Byte(b) => Some(*b as i64),
            NbtTag::Short(s) => Some(*s as i64),
            NbtTag::Int(i) => Some(*i as i64),
            NbtTag::Long(l) => Some(*l),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[NbtTag]> {
        if let NbtTag::List(ref l) = self {
            Some(l.as_slice())
        } else {
            None
        }
    }
}

/// NBT Decoder and Encoder supporting unnamed root (network NBT) and named root (disk NBT).
pub struct Nbt;

impl Nbt {
    /// Read gzip-compressed NBT from bytes (e.g. `level.dat`).
    pub fn from_gzip(bytes: &[u8]) -> Result<(String, NbtTag), NbtError> {
        let mut decoder = GzDecoder::new(bytes);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Self::from_reader(&mut Cursor::new(decompressed))
    }

    /// Write NBT as gzip-compressed bytes.
    pub fn to_gzip(root_name: &str, tag: &NbtTag) -> Result<Vec<u8>, NbtError> {
        let mut uncompressed = Vec::new();
        Self::to_writer(&mut uncompressed, root_name, tag)?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&uncompressed)?;
        Ok(encoder.finish()?)
    }

    /// Read standard disk NBT with root name from reader.
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<(String, NbtTag), NbtError> {
        let tag_id = reader.read_u8()?;
        if tag_id == 0 {
            return Ok((String::new(), NbtTag::End));
        }
        let name_len = reader.read_u16::<BigEndian>()? as usize;
        let mut name_bytes = vec![0u8; name_len];
        reader.read_exact(&mut name_bytes)?;
        let name = String::from_utf8_lossy(&name_bytes).into_owned();

        let tag = Self::read_payload(reader, tag_id)?;
        Ok((name, tag))
    }

    /// Read unnamed NBT tag (used in modern network packets 1.20.2+).
    pub fn from_network<R: Read>(reader: &mut R) -> Result<NbtTag, NbtError> {
        let tag_id = reader.read_u8()?;
        if tag_id == 0 {
            return Ok(NbtTag::End);
        }
        Self::read_payload(reader, tag_id)
    }

    /// Write standard disk NBT with root name to writer.
    pub fn to_writer<W: Write>(writer: &mut W, root_name: &str, tag: &NbtTag) -> Result<(), NbtError> {
        writer.write_u8(tag.id())?;
        if tag.id() != 0 {
            let name_bytes = root_name.as_bytes();
            writer.write_u16::<BigEndian>(name_bytes.len() as u16)?;
            writer.write_all(name_bytes)?;
            Self::write_payload(writer, tag)?;
        }
        Ok(())
    }

    /// Write unnamed NBT tag for network packets.
    pub fn to_network<W: Write>(writer: &mut W, tag: &NbtTag) -> Result<(), NbtError> {
        writer.write_u8(tag.id())?;
        if tag.id() != 0 {
            Self::write_payload(writer, tag)?;
        }
        Ok(())
    }

    fn read_payload<R: Read>(reader: &mut R, tag_id: u8) -> Result<NbtTag, NbtError> {
        match tag_id {
            0 => Ok(NbtTag::End),
            1 => Ok(NbtTag::Byte(reader.read_i8()?)),
            2 => Ok(NbtTag::Short(reader.read_i16::<BigEndian>()?)),
            3 => Ok(NbtTag::Int(reader.read_i32::<BigEndian>()?)),
            4 => Ok(NbtTag::Long(reader.read_i64::<BigEndian>()?)),
            5 => Ok(NbtTag::Float(reader.read_f32::<BigEndian>()?)),
            6 => Ok(NbtTag::Double(reader.read_f64::<BigEndian>()?)),
            7 => {
                let len = reader.read_i32::<BigEndian>()?;
                if len < 0 {
                    return Err(NbtError::ParseError("Negative ByteArray length".into()));
                }
                let mut buf = vec![0u8; len as usize];
                reader.read_exact(&mut buf)?;
                Ok(NbtTag::ByteArray(buf))
            }
            8 => {
                let len = reader.read_u16::<BigEndian>()? as usize;
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf)?;
                Ok(NbtTag::String(String::from_utf8_lossy(&buf).into_owned()))
            }
            9 => {
                let elem_id = reader.read_u8()?;
                let len = reader.read_i32::<BigEndian>()?;
                if len <= 0 {
                    return Ok(NbtTag::List(Vec::new()));
                }
                let mut list = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    list.push(Self::read_payload(reader, elem_id)?);
                }
                Ok(NbtTag::List(list))
            }
            10 => {
                let mut compound = BTreeMap::new();
                loop {
                    let sub_id = reader.read_u8()?;
                    if sub_id == 0 {
                        break;
                    }
                    let name_len = reader.read_u16::<BigEndian>()? as usize;
                    let mut name_bytes = vec![0u8; name_len];
                    reader.read_exact(&mut name_bytes)?;
                    let key = String::from_utf8_lossy(&name_bytes).into_owned();
                    let val = Self::read_payload(reader, sub_id)?;
                    compound.insert(key, val);
                }
                Ok(NbtTag::Compound(compound))
            }
            11 => {
                let len = reader.read_i32::<BigEndian>()?;
                if len < 0 {
                    return Err(NbtError::ParseError("Negative IntArray length".into()));
                }
                let mut arr = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    arr.push(reader.read_i32::<BigEndian>()?);
                }
                Ok(NbtTag::IntArray(arr))
            }
            12 => {
                let len = reader.read_i32::<BigEndian>()?;
                if len < 0 {
                    return Err(NbtError::ParseError("Negative LongArray length".into()));
                }
                let mut arr = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    arr.push(reader.read_i64::<BigEndian>()?);
                }
                Ok(NbtTag::LongArray(arr))
            }
            other => Err(NbtError::InvalidTagId(other)),
        }
    }

    fn write_payload<W: Write>(writer: &mut W, tag: &NbtTag) -> Result<(), NbtError> {
        match tag {
            NbtTag::End => Ok(()),
            NbtTag::Byte(b) => {
                writer.write_i8(*b)?;
                Ok(())
            }
            NbtTag::Short(s) => {
                writer.write_i16::<BigEndian>(*s)?;
                Ok(())
            }
            NbtTag::Int(i) => {
                writer.write_i32::<BigEndian>(*i)?;
                Ok(())
            }
            NbtTag::Long(l) => {
                writer.write_i64::<BigEndian>(*l)?;
                Ok(())
            }
            NbtTag::Float(f) => {
                writer.write_f32::<BigEndian>(*f)?;
                Ok(())
            }
            NbtTag::Double(d) => {
                writer.write_f64::<BigEndian>(*d)?;
                Ok(())
            }
            NbtTag::ByteArray(arr) => {
                writer.write_i32::<BigEndian>(arr.len() as i32)?;
                writer.write_all(arr)?;
                Ok(())
            }
            NbtTag::String(s) => {
                let bytes = s.as_bytes();
                writer.write_u16::<BigEndian>(bytes.len() as u16)?;
                writer.write_all(bytes)?;
                Ok(())
            }
            NbtTag::List(list) => {
                if list.is_empty() {
                    writer.write_u8(0)?;
                    writer.write_i32::<BigEndian>(0)?;
                } else {
                    let elem_id = list[0].id();
                    writer.write_u8(elem_id)?;
                    writer.write_i32::<BigEndian>(list.len() as i32)?;
                    for item in list {
                        Self::write_payload(writer, item)?;
                    }
                }
                Ok(())
            }
            NbtTag::Compound(map) => {
                for (key, val) in map {
                    writer.write_u8(val.id())?;
                    let key_bytes = key.as_bytes();
                    writer.write_u16::<BigEndian>(key_bytes.len() as u16)?;
                    writer.write_all(key_bytes)?;
                    Self::write_payload(writer, val)?;
                }
                writer.write_u8(0)?; // Tag_End
                Ok(())
            }
            NbtTag::IntArray(arr) => {
                writer.write_i32::<BigEndian>(arr.len() as i32)?;
                for &v in arr {
                    writer.write_i32::<BigEndian>(v)?;
                }
                Ok(())
            }
            NbtTag::LongArray(arr) => {
                writer.write_i32::<BigEndian>(arr.len() as i32)?;
                for &v in arr {
                    writer.write_i64::<BigEndian>(v)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nbt_roundtrip() {
        let mut map = BTreeMap::new();
        map.insert("GameVersion".to_string(), NbtTag::String("1.21.11".to_string()));
        map.insert("TickCount".to_string(), NbtTag::Long(123456789));
        map.insert("PlayerPosition".to_string(), NbtTag::List(vec![
            NbtTag::Double(100.5),
            NbtTag::Double(64.0),
            NbtTag::Double(-200.25),
        ]));

        let root = NbtTag::Compound(map);
        let mut buffer = Vec::new();
        Nbt::to_writer(&mut buffer, "Data", &root).expect("write failed");

        let (name, decoded) = Nbt::from_reader(&mut Cursor::new(&buffer)).expect("read failed");
        assert_eq!(name, "Data");
        assert_eq!(decoded, root);
    }
}
