use crate::codec::{PacketReader, PacketWriter};
use std::io;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Play,
}

/// Outgoing Handshake Packet
pub struct HandshakePacket {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32, // 1 for status, 2 for login
}

impl HandshakePacket {
    pub fn encode(&self) -> Vec<u8> {
        let mut writer = PacketWriter::new();
        writer.write_varint(0x00); // Packet ID
        writer.write_varint(self.protocol_version);
        writer.write_string(&self.server_address);
        writer.write_u16(self.server_port);
        writer.write_varint(self.next_state);
        writer.into_inner()
    }
}

/// Outgoing Login Start Packet (1.21.x)
pub struct LoginStartPacket {
    pub username: String,
    pub uuid: Uuid,
}

impl LoginStartPacket {
    pub fn encode(&self) -> Vec<u8> {
        let mut writer = PacketWriter::new();
        writer.write_varint(0x00); // Packet ID
        writer.write_string(&self.username);
        writer.write_uuid(&self.uuid);
        writer.into_inner()
    }
}

/// Incoming Login Success Packet
#[derive(Debug, Clone)]
pub struct LoginSuccessPacket {
    pub uuid: Uuid,
    pub username: String,
}

impl LoginSuccessPacket {
    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = PacketReader::new(payload);
        let _packet_id = reader.read_varint()?;
        let uuid = reader.read_uuid()?;
        let username = reader.read_string()?;
        Ok(Self { uuid, username })
    }
}

/// Incoming Set Compression Packet
#[derive(Debug, Clone)]
pub struct SetCompressionPacket {
    pub threshold: i32,
}

impl SetCompressionPacket {
    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = PacketReader::new(payload);
        let _packet_id = reader.read_varint()?;
        let threshold = reader.read_varint()?;
        Ok(Self { threshold })
    }
}

/// Outgoing Status Request Packet
pub struct StatusRequestPacket;

impl StatusRequestPacket {
    pub fn encode() -> Vec<u8> {
        let mut writer = PacketWriter::new();
        writer.write_varint(0x00);
        writer.into_inner()
    }
}

/// Outgoing Ping Packet
pub struct PingPacket {
    pub payload: i64,
}

impl PingPacket {
    pub fn encode(&self) -> Vec<u8> {
        let mut writer = PacketWriter::new();
        writer.write_varint(0x01);
        writer.write_i64(self.payload);
        writer.into_inner()
    }
}

/// Incoming Status Response Packet
#[derive(Debug, Clone)]
pub struct StatusResponsePacket {
    pub json: String,
}

impl StatusResponsePacket {
    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = PacketReader::new(payload);
        let _packet_id = reader.read_varint()?;
        let json = reader.read_string()?;
        Ok(Self { json })
    }
}

/// Serverbound Teleport Confirm
pub struct TeleportConfirmPacket {
    pub teleport_id: i32,
}

impl TeleportConfirmPacket {
    pub fn encode(&self) -> Vec<u8> {
        let mut writer = PacketWriter::new();
        writer.write_varint(0x00); // 1.21 teleport confirm packet id
        writer.write_varint(self.teleport_id);
        writer.into_inner()
    }
}

/// Serverbound KeepAlive
pub struct ServerboundKeepAlivePacket {
    pub id: i64,
}

impl ServerboundKeepAlivePacket {
    pub fn encode(&self, packet_id: i32) -> Vec<u8> {
        let mut writer = PacketWriter::new();
        writer.write_varint(packet_id);
        writer.write_i64(self.id);
        writer.into_inner()
    }
}

/// Serverbound Player Position and Rotation
pub struct PlayerPositionAndRotationPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl PlayerPositionAndRotationPacket {
    pub fn encode(&self, packet_id: i32) -> Vec<u8> {
        let mut writer = PacketWriter::new();
        writer.write_varint(packet_id);
        writer.write_f64(self.x);
        writer.write_f64(self.y);
        writer.write_f64(self.z);
        writer.write_f32(self.yaw);
        writer.write_f32(self.pitch);
        writer.write_bool(self.on_ground);
        writer.into_inner()
    }
}
