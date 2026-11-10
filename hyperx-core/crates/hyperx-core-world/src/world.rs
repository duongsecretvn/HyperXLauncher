use crate::chunk::ChunkColumn;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub type ChunkPos = (i32, i32);

#[derive(Default)]
pub struct World {
    chunks: RwLock<HashMap<ChunkPos, Arc<RwLock<ChunkColumn>>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert_chunk(&self, chunk_x: i32, chunk_z: i32, column: ChunkColumn) {
        let mut map = self.chunks.write();
        map.insert((chunk_x, chunk_z), Arc::new(RwLock::new(column)));
    }

    pub fn remove_chunk(&self, chunk_x: i32, chunk_z: i32) {
        let mut map = self.chunks.write();
        map.remove(&(chunk_x, chunk_z));
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> u32 {
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        let map = self.chunks.read();
        if let Some(chunk_arc) = map.get(&(chunk_x, chunk_z)) {
            let chunk = chunk_arc.read();
            chunk.get_block(x, y, z)
        } else {
            0
        }
    }

    pub fn set_block(&self, x: i32, y: i32, z: i32, state: u32) {
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        let map = self.chunks.read();
        if let Some(chunk_arc) = map.get(&(chunk_x, chunk_z)) {
            let mut chunk = chunk_arc.write();
            chunk.set_block(x, y, z, state);
        }
    }

    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<Arc<RwLock<ChunkColumn>>> {
        let map = self.chunks.read();
        map.get(&(chunk_x, chunk_z)).cloned()
    }

    pub fn loaded_chunk_count(&self) -> usize {
        let map = self.chunks.read();
        map.len()
    }
}
