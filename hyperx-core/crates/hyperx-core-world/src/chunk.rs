use crate::palette::PalettedContainer;

pub const SECTION_SIZE: usize = 16;
pub const SECTIONS_PER_CHUNK: usize = 24; // Y: -64 to 319
pub const MIN_Y: i32 = -64;
pub const MAX_Y: i32 = 320;

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub block_states: PalettedContainer,
    pub block_count: u16,
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self {
            block_states: PalettedContainer::new_empty(),
            block_count: 0,
        }
    }
}

impl ChunkSection {
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> u32 {
        let index = (y << 8) | (z << 4) | x;
        self.block_states.get(index)
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, state: u32) {
        let index = (y << 8) | (z << 4) | x;
        let old = self.block_states.get(index);
        if old == 0 && state != 0 {
            self.block_count += 1;
        } else if old != 0 && state == 0 {
            self.block_count = self.block_count.saturating_sub(1);
        }
        self.block_states.set(index, state);
    }
}

#[derive(Debug, Clone)]
pub struct ChunkColumn {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub sections: Vec<ChunkSection>,
}

impl ChunkColumn {
    pub fn new(chunk_x: i32, chunk_z: i32) -> Self {
        Self {
            chunk_x,
            chunk_z,
            sections: (0..SECTIONS_PER_CHUNK).map(|_| ChunkSection::default()).collect(),
        }
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> u32 {
        if !(MIN_Y..MAX_Y).contains(&y) {
            return 0;
        }
        let local_x = (x.rem_euclid(16)) as usize;
        let local_z = (z.rem_euclid(16)) as usize;
        let section_idx = ((y - MIN_Y) / 16) as usize;
        let local_y = ((y - MIN_Y).rem_euclid(16)) as usize;

        if let Some(section) = self.sections.get(section_idx) {
            section.get_block(local_x, local_y, local_z)
        } else {
            0
        }
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, state: u32) {
        if !(MIN_Y..MAX_Y).contains(&y) {
            return;
        }
        let local_x = (x.rem_euclid(16)) as usize;
        let local_z = (z.rem_euclid(16)) as usize;
        let section_idx = ((y - MIN_Y) / 16) as usize;
        let local_y = ((y - MIN_Y).rem_euclid(16)) as usize;

        if let Some(section) = self.sections.get_mut(section_idx) {
            section.set_block(local_x, local_y, local_z, state);
        }
    }
}
