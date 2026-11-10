//! Minecraft UUID conversions and offline player UUID synthesis.

use uuid::Uuid;

/// Convert a modern Minecraft IntArray UUID `[i32; 4]` to standard `uuid::Uuid`.
#[inline]
pub fn uuid_from_int_array(arr: [i32; 4]) -> Uuid {
    let mut bytes = [0u8; 16];
    for (i, &int_val) in arr.iter().enumerate() {
        bytes[i * 4..(i + 1) * 4].copy_from_slice(&(int_val as u32).to_be_bytes());
    }
    Uuid::from_bytes(bytes)
}

/// Convert a `uuid::Uuid` to Minecraft IntArray `[i32; 4]`.
#[inline]
pub fn uuid_to_int_array(u: &Uuid) -> [i32; 4] {
    let bytes = u.as_bytes();
    let mut arr = [0i32; 4];
    for (i, chunk) in bytes.chunks_exact(4).enumerate() {
        arr[i] = u32::from_be_bytes(chunk.try_into().unwrap()) as i32;
    }
    arr
}

/// Generate offline-mode player UUID as Java Minecraft does:
/// `UUID.nameUUIDFromBytes(("OfflinePlayer:" + username).getBytes(StandardCharsets.UTF_8))` (UUID v3).
pub fn offline_player_uuid(username: &str) -> Uuid {
    let input = format!("OfflinePlayer:{}", username);
    Uuid::new_v3(&Uuid::nil(), input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_roundtrip() {
        let u = Uuid::new_v4();
        let arr = uuid_to_int_array(&u);
        let recovered = uuid_from_int_array(arr);
        assert_eq!(u, recovered);
    }

    #[test]
    fn test_offline_uuid() {
        let u = offline_player_uuid("duongsecretVN");
        assert_eq!(u.get_version_num(), 3);
    }
}
