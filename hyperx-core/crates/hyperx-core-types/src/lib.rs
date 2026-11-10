//! `hyperx-core-types`: Essential Minecraft primitives, NBT, SNBT, VarInt, UUID, and Text Components.

pub mod identifier;
pub mod nbt;
pub mod snbt;
pub mod text;
pub mod uuid_util;
pub mod varint;

pub use identifier::Identifier;
pub use nbt::{Compound, Nbt, NbtError, NbtTag};
pub use snbt::{parse_snbt, SnbtError};
pub use text::{StructuredComponent, TextComponent};
pub use uuid_util::{offline_player_uuid, uuid_from_int_array, uuid_to_int_array};
pub use varint::{
    decode_varint_slice, decode_varlong_slice, encode_varint_slice, read_varint, read_varlong,
    varint_size, write_varint, write_varlong, VarIntError,
};
