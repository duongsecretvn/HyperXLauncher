use glam::Vec2;
use std::collections::HashMap;

/// Texture UV coordinates within an atlas
#[derive(Debug, Clone, Copy, Default)]
pub struct AtlasSubTexture {
    pub min: Vec2,
    pub max: Vec2,
}

/// Dynamic texture atlas builder
#[derive(Debug, Default)]
pub struct TextureAtlas {
    pub width: u32,
    pub height: u32,
    pub textures: HashMap<String, AtlasSubTexture>,
    pub rgba_data: Vec<u8>,
}

impl TextureAtlas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            textures: HashMap::new(),
            rgba_data: vec![0; (width * height * 4) as usize],
        }
    }

    pub fn insert_texture(&mut self, id: String, x: u32, y: u32, w: u32, h: u32, data: &[u8]) {
        for row in 0..h {
            let src_start = (row * w * 4) as usize;
            let src_end = src_start + (w * 4) as usize;
            let dst_start = (((y + row) * self.width + x) * 4) as usize;
            let dst_end = dst_start + (w * 4) as usize;

            if src_end <= data.len() && dst_end <= self.rgba_data.len() {
                self.rgba_data[dst_start..dst_end].copy_from_slice(&data[src_start..src_end]);
            }
        }

        let min = Vec2::new(x as f32 / self.width as f32, y as f32 / self.height as f32);
        let max = Vec2::new(
            (x + w) as f32 / self.width as f32,
            (y + h) as f32 / self.height as f32,
        );

        self.textures.insert(id, AtlasSubTexture { min, max });
    }

    pub fn get_subtexture(&self, id: &str) -> Option<&AtlasSubTexture> {
        self.textures.get(id)
    }
}
