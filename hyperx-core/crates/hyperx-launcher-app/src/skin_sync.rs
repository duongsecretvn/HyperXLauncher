use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkinSyncError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image error: {0}")]
    Image(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinCapeData {
    pub username: String,
    pub has_skin: bool,
    pub skin_base64: Option<String>,
    pub has_cape: bool,
    pub cape_base64: Option<String>,
}

pub struct SkinSynchronizer {
    http: reqwest::Client,
    cache_dir: PathBuf,
}

impl SkinSynchronizer {
    pub fn new(cache_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&cache_dir);
        Self {
            http: reqwest::Client::builder()
                .user_agent("HyperXLauncher-SkinSync/1.0.0")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            cache_dir,
        }
    }

    /// Fetches and caches Ely.by skin and cape textures, converting to base64 data URLs
    pub async fn fetch_textures(&self, username: &str) -> Result<SkinCapeData, SkinSyncError> {
        let skin_url = format!("http://skinsystem.ely.by/textures/{}", username);
        let cape_url = format!("http://skinsystem.ely.by/cloaks/{}", username);

        let skin_cache_path = self.cache_dir.join(format!("{}_skin.png", username));
        let cape_cache_path = self.cache_dir.join(format!("{}_cape.png", username));

        // 1. Fetch skin
        let skin_res = self.http.get(&skin_url).send().await;
        let mut skin_base64 = None;
        let mut has_skin = false;

        if let Ok(res) = skin_res {
            if res.status().is_success() {
                if let Ok(bytes) = res.bytes().await {
                    if !bytes.is_empty() {
                        let _ = std::fs::write(&skin_cache_path, &bytes);
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                        skin_base64 = Some(format!("data:image/png;base64,{}", b64));
                        has_skin = true;
                    }
                }
            }
        }

        // Fallback to cache if network fails
        if !has_skin && skin_cache_path.exists() {
            if let Ok(bytes) = std::fs::read(&skin_cache_path) {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                skin_base64 = Some(format!("data:image/png;base64,{}", b64));
                has_skin = true;
            }
        }

        // 2. Fetch cape
        let cape_res = self.http.get(&cape_url).send().await;
        let mut cape_base64 = None;
        let mut has_cape = false;

        if let Ok(res) = cape_res {
            if res.status().is_success() {
                if let Ok(bytes) = res.bytes().await {
                    if !bytes.is_empty() {
                        let _ = std::fs::write(&cape_cache_path, &bytes);
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                        cape_base64 = Some(format!("data:image/png;base64,{}", b64));
                        has_cape = true;
                    }
                }
            }
        }

        // Fallback to cache if network fails
        if !has_cape && cape_cache_path.exists() {
            if let Ok(bytes) = std::fs::read(&cape_cache_path) {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                cape_base64 = Some(format!("data:image/png;base64,{}", b64));
                has_cape = true;
            }
        }

        Ok(SkinCapeData {
            username: username.to_string(),
            has_skin,
            skin_base64,
            has_cape,
            cape_base64,
        })
    }
}
