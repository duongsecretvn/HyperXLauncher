//! Network protocol layer for HyperXCore (Handshake, Login, Play).
pub mod codec;
pub mod compression;
pub mod crypto;
pub mod packet;
pub mod client;

pub use client::MinecraftClient;
