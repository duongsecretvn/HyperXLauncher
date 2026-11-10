use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use hyperx_core_types::varint::{read_varint, write_varint};
use std::io::{self, Cursor, Read};
use uuid::Uuid;

/// Zero-copy / minimal alloc Packet Buffer Reader
pub struct PacketReader<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> PacketReader<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(slice),
        }
    }

    pub fn read_varint(&mut self) -> io::Result<i32> {
        read_varint(&mut self.cursor).map_err(Into::into)
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        let b = self.cursor.read_u8()?;
        Ok(b != 0)
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        self.cursor.read_u8()
    }

    pub fn read_i8(&mut self) -> io::Result<i8> {
        self.cursor.read_i8()
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        self.cursor.read_u16::<BigEndian>()
    }

    pub fn read_i16(&mut self) -> io::Result<i16> {
        self.cursor.read_i16::<BigEndian>()
    }

    pub fn read_i32(&mut self) -> io::Result<i32> {
        self.cursor.read_i32::<BigEndian>()
    }

    pub fn read_i64(&mut self) -> io::Result<i64> {
        self.cursor.read_i64::<BigEndian>()
    }

    pub fn read_f32(&mut self) -> io::Result<f32> {
        self.cursor.read_f32::<BigEndian>()
    }

    pub fn read_f64(&mut self) -> io::Result<f64> {
        self.cursor.read_f64::<BigEndian>()
    }

    pub fn read_string(&mut self) -> io::Result<String> {
        let len = self.read_varint()? as usize;
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn read_uuid(&mut self) -> io::Result<Uuid> {
        let mut bytes = [0u8; 16];
        self.cursor.read_exact(&mut bytes)?;
        Ok(Uuid::from_bytes(bytes))
    }

    pub fn read_byte_array(&mut self) -> io::Result<Vec<u8>> {
        let len = self.read_varint()? as usize;
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn remaining_bytes(&self) -> &'a [u8] {
        let pos = self.cursor.position() as usize;
        let slice = self.cursor.get_ref();
        if pos < slice.len() {
            &slice[pos..]
        } else {
            &[]
        }
    }
}

/// Dynamic Packet Buffer Writer
#[derive(Default)]
pub struct PacketWriter {
    buf: Vec<u8>,
}

impl PacketWriter {
    pub fn new() -> Self {
        Self { buf: Vec::with_capacity(256) }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { buf: Vec::with_capacity(capacity) }
    }

    pub fn write_varint(&mut self, val: i32) {
        let _ = write_varint(&mut self.buf, val);
    }

    pub fn write_bool(&mut self, val: bool) {
        self.buf.push(if val { 1 } else { 0 });
    }

    pub fn write_u8(&mut self, val: u8) {
        self.buf.push(val);
    }

    pub fn write_i8(&mut self, val: i8) {
        self.buf.push(val as u8);
    }

    pub fn write_u16(&mut self, val: u16) {
        let _ = self.buf.write_u16::<BigEndian>(val);
    }

    pub fn write_i16(&mut self, val: i16) {
        let _ = self.buf.write_i16::<BigEndian>(val);
    }

    pub fn write_i32(&mut self, val: i32) {
        let _ = self.buf.write_i32::<BigEndian>(val);
    }

    pub fn write_i64(&mut self, val: i64) {
        let _ = self.buf.write_i64::<BigEndian>(val);
    }

    pub fn write_f32(&mut self, val: f32) {
        let _ = self.buf.write_f32::<BigEndian>(val);
    }

    pub fn write_f64(&mut self, val: f64) {
        let _ = self.buf.write_f64::<BigEndian>(val);
    }

    pub fn write_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.buf.extend_from_slice(bytes);
    }

    pub fn write_uuid(&mut self, uuid: &Uuid) {
        self.buf.extend_from_slice(uuid.as_bytes());
    }

    pub fn write_byte_array(&mut self, bytes: &[u8]) {
        self.write_varint(bytes.len() as i32);
        self.buf.extend_from_slice(bytes);
    }

    pub fn write_raw_bytes(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}
