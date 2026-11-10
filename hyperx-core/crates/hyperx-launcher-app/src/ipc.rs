use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

use crate::auth::AuthManager;
use crate::downloader::ParallelDownloader;
use crate::runner::{BedrockLaunchOptions, GameRunner, JavaLaunchOptions};
use crate::skin_sync::SkinSynchronizer;
use crate::updater::Updater;

#[derive(Debug, Deserialize)]
pub struct IpcRequest {
    pub id: u64,
    pub command: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct IpcResponse {
    pub id: u64,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSpecs {
    pub os: String,
    pub arch: String,
    pub total_memory_mb: u64,
    pub cpu_cores: usize,
    pub rust_core_version: String,
}

pub struct IpcDispatcher {
    runner: Arc<GameRunner>,
    auth: Arc<AuthManager>,
    skin_sync: Arc<SkinSynchronizer>,
    #[allow(dead_code)]
    downloader: Arc<ParallelDownloader>,
    updater: Arc<Updater>,
}

impl IpcDispatcher {
    pub fn new(app_dir: PathBuf) -> Self {
        Self {
            runner: Arc::new(GameRunner::new()),
            auth: Arc::new(AuthManager::new()),
            skin_sync: Arc::new(SkinSynchronizer::new(app_dir.join("skins"))),
            downloader: Arc::new(ParallelDownloader::new(16)),
            updater: Arc::new(Updater::new("1.0.0")),
        }
    }

    pub fn is_game_running(&self) -> bool {
        self.runner.is_running()
    }

    pub async fn dispatch(&self, req: IpcRequest) -> IpcResponse {
        match req.command.as_str() {
            "launch_java" => match serde_json::from_value::<JavaLaunchOptions>(req.payload) {
                Ok(opts) => match self.runner.launch_java(opts) {
                    Ok(pid) => IpcResponse {
                        id: req.id,
                        success: true,
                        data: Some(serde_json::json!({ "pid": pid })),
                        error: None,
                    },
                    Err(e) => IpcResponse {
                        id: req.id,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    },
                },
                Err(e) => IpcResponse {
                    id: req.id,
                    success: false,
                    data: None,
                    error: Some(format!("Invalid Java launch payload: {}", e)),
                },
            },

            "launch_bedrock" => match serde_json::from_value::<BedrockLaunchOptions>(req.payload) {
                Ok(opts) => match self.runner.launch_bedrock(opts) {
                    Ok(pid) => IpcResponse {
                        id: req.id,
                        success: true,
                        data: Some(serde_json::json!({ "pid": pid })),
                        error: None,
                    },
                    Err(e) => IpcResponse {
                        id: req.id,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    },
                },
                Err(e) => IpcResponse {
                    id: req.id,
                    success: false,
                    data: None,
                    error: Some(format!("Invalid Bedrock launch payload: {}", e)),
                },
            },

            "auth_elyby" => {
                #[derive(Deserialize)]
                struct LoginPayload {
                    username: String,
                    password: String,
                }
                match serde_json::from_value::<LoginPayload>(req.payload) {
                    Ok(p) => match self.auth.login_elyby(&p.username, &p.password).await {
                        Ok(session) => IpcResponse {
                            id: req.id,
                            success: true,
                            data: Some(serde_json::to_value(session).unwrap_or_default()),
                            error: None,
                        },
                        Err(e) => IpcResponse {
                            id: req.id,
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                        },
                    },
                    Err(e) => IpcResponse {
                        id: req.id,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    },
                }
            }

            "auth_offline" => {
                #[derive(Deserialize)]
                struct OfflinePayload {
                    username: String,
                }
                match serde_json::from_value::<OfflinePayload>(req.payload) {
                    Ok(p) => {
                        let session = self.auth.login_offline(&p.username);
                        IpcResponse {
                            id: req.id,
                            success: true,
                            data: Some(serde_json::to_value(session).unwrap_or_default()),
                            error: None,
                        }
                    }
                    Err(e) => IpcResponse {
                        id: req.id,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    },
                }
            }

            "sync_skin" => {
                #[derive(Deserialize)]
                struct SkinPayload {
                    username: String,
                }
                match serde_json::from_value::<SkinPayload>(req.payload) {
                    Ok(p) => match self.skin_sync.fetch_textures(&p.username).await {
                        Ok(data) => IpcResponse {
                            id: req.id,
                            success: true,
                            data: Some(serde_json::to_value(data).unwrap_or_default()),
                            error: None,
                        },
                        Err(e) => IpcResponse {
                            id: req.id,
                            success: false,
                            data: None,
                            error: Some(e.to_string()),
                        },
                    },
                    Err(e) => IpcResponse {
                        id: req.id,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    },
                }
            }

            "check_update" => match self.updater.check_for_updates().await {
                Ok(res) => IpcResponse {
                    id: req.id,
                    success: true,
                    data: Some(serde_json::to_value(res).unwrap_or_default()),
                    error: None,
                },
                Err(e) => IpcResponse {
                    id: req.id,
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                },
            },

            "get_system_specs" => {
                let specs = SystemSpecs {
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    total_memory_mb: 16384, // Standard or detected
                    cpu_cores: std::thread::available_parallelism()
                        .map(|n| n.get())
                        .unwrap_or(4),
                    rust_core_version: "1.0.0 (Native Rust Engine)".to_string(),
                };
                IpcResponse {
                    id: req.id,
                    success: true,
                    data: Some(serde_json::to_value(specs).unwrap_or_default()),
                    error: None,
                }
            }

            _ => IpcResponse {
                id: req.id,
                success: false,
                data: None,
                error: Some(format!("Unknown command: {}", req.command)),
            },
        }
    }
}
