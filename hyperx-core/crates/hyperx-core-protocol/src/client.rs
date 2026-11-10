use crate::compression::{compress_zlib, decompress_zlib};
use crate::crypto::AesCfb8;
use crate::packet::{
    ConnectionState, HandshakePacket, LoginStartPacket, LoginSuccessPacket,
    SetCompressionPacket,
};
use hyperx_core_types::varint::{read_varint, write_varint};
use std::io::{self, Cursor};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;

pub struct MinecraftClient {
    stream: TcpStream,
    state: ConnectionState,
    compression_threshold: Option<i32>,
    cipher_encrypt: Option<AesCfb8>,
    cipher_decrypt: Option<AesCfb8>,
}

impl MinecraftClient {
    pub async fn connect(host: &str, port: u16) -> io::Result<Self> {
        let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
        Ok(Self {
            stream,
            state: ConnectionState::Handshake,
            compression_threshold: None,
            cipher_encrypt: None,
            cipher_decrypt: None,
        })
    }

    pub fn set_encryption(&mut self, key: &[u8; 16]) {
        self.cipher_encrypt = Some(AesCfb8::new(key, key));
        self.cipher_decrypt = Some(AesCfb8::new(key, key));
    }

    pub async fn send_packet(&mut self, payload: &[u8]) -> io::Result<()> {
        let mut framed = Vec::new();

        if let Some(threshold) = self.compression_threshold {
            if payload.len() as i32 >= threshold {
                let compressed = compress_zlib(payload)?;
                let mut data_len_buf = Vec::new();
                write_varint(&mut data_len_buf, payload.len() as i32)?;

                let packet_len = (data_len_buf.len() + compressed.len()) as i32;
                write_varint(&mut framed, packet_len)?;
                framed.extend_from_slice(&data_len_buf);
                framed.extend_from_slice(&compressed);
            } else {
                let mut zero_buf = Vec::new();
                write_varint(&mut zero_buf, 0)?;

                let packet_len = (zero_buf.len() + payload.len()) as i32;
                write_varint(&mut framed, packet_len)?;
                framed.extend_from_slice(&zero_buf);
                framed.extend_from_slice(payload);
            }
        } else {
            write_varint(&mut framed, payload.len() as i32)?;
            framed.extend_from_slice(payload);
        }

        if let Some(enc) = &mut self.cipher_encrypt {
            enc.encrypt(&mut framed);
        }

        self.stream.write_all(&framed).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn read_raw_byte(&mut self) -> io::Result<u8> {
        let mut b = [0u8; 1];
        self.stream.read_exact(&mut b).await?;
        if let Some(dec) = &mut self.cipher_decrypt {
            dec.decrypt(&mut b);
        }
        Ok(b[0])
    }

    pub async fn read_varint(&mut self) -> io::Result<i32> {
        let mut res = 0i32;
        let mut count = 0;
        loop {
            let byte = self.read_raw_byte().await?;
            let val = (byte & 0x7F) as i32;
            res |= val << (7 * count);
            count += 1;
            if count > 5 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too long"));
            }
            if (byte & 0x80) == 0 {
                break;
            }
        }
        Ok(res)
    }

    pub async fn read_packet(&mut self) -> io::Result<Vec<u8>> {
        let packet_length = self.read_varint().await? as usize;
        let mut raw_buf = vec![0u8; packet_length];
        self.stream.read_exact(&mut raw_buf).await?;
        if let Some(dec) = &mut self.cipher_decrypt {
            dec.decrypt(&mut raw_buf);
        }

        if self.compression_threshold.is_some() {
            let mut cursor = Cursor::new(&raw_buf);
            let uncompressed_len = read_varint(&mut cursor).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))? as usize;
            let offset = cursor.position() as usize;

            if uncompressed_len == 0 {
                Ok(raw_buf[offset..].to_vec())
            } else {
                let decompressed = decompress_zlib(&raw_buf[offset..], uncompressed_len)?;
                Ok(decompressed)
            }
        } else {
            Ok(raw_buf)
        }
    }

    /// Perform standard Minecraft 1.21.x handshake and offline login sequence
    pub async fn handshake_and_login(
        &mut self,
        protocol_version: i32,
        host: &str,
        port: u16,
        username: &str,
        uuid: &Uuid,
    ) -> io::Result<LoginSuccessPacket> {
        // Send Handshake
        let handshake = HandshakePacket {
            protocol_version,
            server_address: host.to_string(),
            server_port: port,
            next_state: 2, // Login
        };
        self.send_packet(&handshake.encode()).await?;
        self.state = ConnectionState::Login;

        // Send Login Start
        let login_start = LoginStartPacket {
            username: username.to_string(),
            uuid: *uuid,
        };
        self.send_packet(&login_start.encode()).await?;

        // Read login responses until Login Success
        loop {
            let packet = self.read_packet().await?;
            if packet.is_empty() {
                continue;
            }
            let packet_id = packet[0];

            match packet_id {
                0x02 => {
                    // Login Success
                    let success = LoginSuccessPacket::decode(&packet)?;
                    self.state = ConnectionState::Play;
                    log::info!("Successfully joined as {} ({})", success.username, success.uuid);
                    return Ok(success);
                }
                0x03 => {
                    // Set Compression
                    let comp = SetCompressionPacket::decode(&packet)?;
                    log::info!("Server enabled packet compression with threshold {}", comp.threshold);
                    self.compression_threshold = Some(comp.threshold);
                }
                0x00 => {
                    // Disconnect (Login)
                    return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Server disconnected login"));
                }
                _ => {
                    log::debug!("Ignored login packet 0x{:02X}", packet_id);
                }
            }
        }
    }
}
