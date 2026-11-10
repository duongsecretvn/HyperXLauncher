use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("No release asset found")]
    NoAssetFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    pub html_url: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: String,
    pub download_url: Option<String>,
}

pub struct Updater {
    client: reqwest::Client,
    current_version: String,
}

impl Updater {
    pub fn new(current_version: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("HyperXLauncher-NativeUpdater/1.0.0")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            current_version: current_version.into(),
        }
    }

    /// Queries the latest release from the official repository duongsecretvn/HyperXLauncher
    pub async fn check_for_updates(&self) -> Result<UpdateCheckResult, UpdateError> {
        let url = "https://api.github.com/repos/duongsecretvn/HyperXLauncher/releases/latest";
        let res = self.client.get(url).send().await?;

        if !res.status().is_success() {
            // If repository has no releases yet, fallback to no update
            return Ok(UpdateCheckResult {
                has_update: false,
                current_version: self.current_version.clone(),
                latest_version: self.current_version.clone(),
                release_notes: "Launcher is up to date".to_string(),
                download_url: None,
            });
        }

        let release: GitHubRelease = res.json().await?;
        let latest_ver = release.tag_name.trim_start_matches('v');

        let has_update = latest_ver != self.current_version;

        let download_url = release
            .assets
            .iter()
            .find(|a| {
                if cfg!(target_os = "windows") {
                    a.name.ends_with(".exe") || a.name.ends_with(".zip")
                } else if cfg!(target_os = "macos") {
                    a.name.ends_with(".dmg") || a.name.ends_with(".app.tar.gz")
                } else {
                    a.name.ends_with(".AppImage") || a.name.ends_with(".tar.gz")
                }
            })
            .map(|a| a.browser_download_url.clone())
            .or_else(|| release.assets.first().map(|a| a.browser_download_url.clone()));

        Ok(UpdateCheckResult {
            has_update,
            current_version: self.current_version.clone(),
            latest_version: latest_ver.to_string(),
            release_notes: release.body.unwrap_or_default(),
            download_url,
        })
    }
}
