use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minecraft BlockState definition JSON format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStateDefinition {
    pub variants: Option<HashMap<String, BlockStateVariant>>,
    pub multipart: Option<Vec<MultipartRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockStateVariant {
    Single(ModelVariant),
    Multiple(Vec<ModelVariant>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVariant {
    pub model: String,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub uvlock: bool,
    #[serde(default = "default_weight")]
    pub weight: u32,
}

fn default_weight() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartRule {
    pub when: Option<serde_json::Value>,
    pub apply: BlockStateVariant,
}

/// Minecraft Block/Item Model JSON format
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawModel {
    pub parent: Option<String>,
    pub ambientocclusion: Option<bool>,
    pub textures: Option<HashMap<String, String>>,
    pub elements: Option<Vec<RawElement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawElement {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: Option<ElementRotation>,
    #[serde(default = "default_shade")]
    pub shade: bool,
    pub faces: HashMap<String, RawFace>,
}

fn default_shade() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRotation {
    pub origin: [f32; 3],
    pub axis: String,
    pub angle: f32,
    #[serde(default)]
    pub rescale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFace {
    pub uv: Option<[f32; 4]>,
    pub texture: String,
    pub cullface: Option<String>,
    #[serde(default)]
    pub rotation: i32,
    #[serde(default)]
    pub tintindex: Option<i32>,
}
