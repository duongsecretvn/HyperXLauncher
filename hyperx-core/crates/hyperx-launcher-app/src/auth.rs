use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Network request error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Authentication failed: {0}")]
    InvalidCredentials(String),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Account not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountType {
    Elyby,
    Microsoft,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub skin_url: Option<String>,
    pub cape_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSession {
    pub account_type: AccountType,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    pub profile: UserProfile,
}

#[derive(Debug, Serialize, Deserialize)]
struct ElybyAuthRequest<'a> {
    username: &'a str,
    password: &'a str,
    #[serde(rename = "clientToken")]
    client_token: &'a str,
    #[serde(rename = "requestUser")]
    request_user: bool,
}

#[derive(Debug, Deserialize)]
struct ElybyProfileResponse {
    id: String,
    name: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ElybyAuthResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "clientToken")]
    client_token: String,
    #[serde(rename = "selectedProfile")]
    selected_profile: Option<ElybyProfileResponse>,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
}

pub struct AuthManager {
    http: reqwest::Client,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("HyperXLauncher/1.0.0 (Native Rust)")
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Authenticates with Ely.by via their standard Yggdrasil-compatible authentication endpoint
    pub async fn login_elyby(&self, username: &str, password: &str) -> Result<AccountSession, AuthError> {
        let client_token = Uuid::new_v4().to_string();
        let payload = ElybyAuthRequest {
            username,
            password,
            client_token: &client_token,
            request_user: true,
        };

        let res = self
            .http
            .post("https://authserver.ely.by/auth/authenticate")
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(AuthError::InvalidCredentials(format!(
                "Ely.by server returned error: {}",
                error_text
            )));
        }

        let body: ElybyAuthResponse = res.json().await?;
        if let Some(err) = body.error_message {
            return Err(AuthError::InvalidCredentials(err));
        }

        let (uuid, name) = if let Some(prof) = body.selected_profile {
            (prof.id, prof.name)
        } else {
            (Uuid::new_v4().to_string(), username.to_string())
        };

        let profile = UserProfile {
            id: uuid.clone(),
            name: name.clone(),
            skin_url: Some(format!("http://skinsystem.ely.by/textures/{}", name)),
            cape_url: Some(format!("http://skinsystem.ely.by/cloaks/{}", name)),
        };

        Ok(AccountSession {
            account_type: AccountType::Elyby,
            username: name,
            uuid,
            access_token: body.access_token,
            refresh_token: Some(client_token),
            expires_at: Some(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() + 86400 * 30),
            profile,
        })
    }

    /// Generates standard offline / local account session with deterministic UUID v3
    pub fn login_offline(&self, username: &str) -> AccountSession {
        // Minecraft offline mode uses UUID v3 computed from "OfflinePlayer:" + username
        let uuid = Uuid::new_v3(&Uuid::NAMESPACE_DNS, format!("OfflinePlayer:{}", username).as_bytes())
            .to_string();

        AccountSession {
            account_type: AccountType::Offline,
            username: username.to_string(),
            uuid: uuid.clone(),
            access_token: Uuid::new_v4().to_string(),
            refresh_token: None,
            expires_at: None,
            profile: UserProfile {
                id: uuid,
                name: username.to_string(),
                skin_url: None,
                cape_url: None,
            },
        }
    }
}
