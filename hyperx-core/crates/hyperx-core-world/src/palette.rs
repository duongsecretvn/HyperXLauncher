/// Bit-packed Paletted Container for 16x16x16 Chunk Sections (4096 blocks)
#[derive(Debug, Clone)]
pub enum PalettedContainer {
    Single(u32),
    Indirect {
        bits_per_entry: u8,
        palette: Vec<u32>,
        data: Vec<u64>,
    },
    Direct {
        bits_per_entry: u8,
        data: Vec<u64>,
    },
}

impl PalettedContainer {
    pub fn new_empty() -> Self {
        Self::Single(0)
    }

    pub fn get(&self, index: usize) -> u32 {
        if index >= 4096 {
            return 0;
        }

        match self {
            Self::Single(val) => *val,
            Self::Indirect {
                bits_per_entry,
                palette,
                data,
            } => {
                let bits = *bits_per_entry as usize;
                if bits == 0 {
                    return palette.first().copied().unwrap_or(0);
                }
                let entries_per_long = 64 / bits;
                let long_idx = index / entries_per_long;
                let sub_idx = index % entries_per_long;
                let bit_offset = sub_idx * bits;
                let mask = (1u64 << bits) - 1;

                if long_idx < data.len() {
                    let palette_idx = ((data[long_idx] >> bit_offset) & mask) as usize;
                    palette.get(palette_idx).copied().unwrap_or(0)
                } else {
                    0
                }
            }
            Self::Direct {
                bits_per_entry,
                data,
            } => {
                let bits = *bits_per_entry as usize;
                let entries_per_long = 64 / bits;
                let long_idx = index / entries_per_long;
                let sub_idx = index % entries_per_long;
                let bit_offset = sub_idx * bits;
                let mask = (1u64 << bits) - 1;

                if long_idx < data.len() {
                    ((data[long_idx] >> bit_offset) & mask) as u32
                } else {
                    0
                }
            }
        }
    }

    pub fn set(&mut self, index: usize, value: u32) {
        if index >= 4096 {
            return;
        }
        match self {
            Self::Single(cur) => {
                if *cur == value {
                    return;
                }
                // Upgrade to Indirect with 4 bits
                let mut data = vec![0u64; 4096 / 16]; // 16 entries per 64-bit long
                let palette = vec![*cur, value];
                // Update element at index
                let long_idx = index / 16;
                let sub_idx = index % 16;
                data[long_idx] |= 1u64 << (sub_idx * 4);
                *self = Self::Indirect {
                    bits_per_entry: 4,
                    palette,
                    data,
                };
            }
            Self::Indirect {
                bits_per_entry,
                palette,
                data,
            } => {
                let bits = *bits_per_entry as usize;
                let p_idx = if let Some(pos) = palette.iter().position(|&v| v == value) {
                    pos
                } else {
                    palette.push(value);
                    palette.len() - 1
                };

                let entries_per_long = 64 / bits;
                let long_idx = index / entries_per_long;
                let sub_idx = index % entries_per_long;
                let bit_offset = sub_idx * bits;
                let mask = (1u64 << bits) - 1;

                if long_idx < data.len() {
                    data[long_idx] = (data[long_idx] & !(mask << bit_offset))
                        | ((p_idx as u64 & mask) << bit_offset);
                }
            }
            Self::Direct {
                bits_per_entry,
                data,
            } => {
                let bits = *bits_per_entry as usize;
                let entries_per_long = 64 / bits;
                let long_idx = index / entries_per_long;
                let sub_idx = index % entries_per_long;
                let bit_offset = sub_idx * bits;
                let mask = (1u64 << bits) - 1;

                if long_idx < data.len() {
                    data[long_idx] = (data[long_idx] & !(mask << bit_offset))
                        | ((value as u64 & mask) << bit_offset);
                }
            }
        }
    }
}
