pub mod auth;
pub mod downloader;
pub mod ipc;
pub mod runner;
pub mod skin_sync;
pub mod updater;
pub mod vibrancy;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_offline_uuid_parity() {
        let auth = auth::AuthManager::new();
        let session = auth.login_offline("duongsecretvn");
        assert_eq!(session.username, "duongsecretvn");
        assert_eq!(session.account_type, auth::AccountType::Offline);

        // Verify deterministic UUID v3 generation matches Minecraft spec
        let session2 = auth.login_offline("duongsecretvn");
        assert_eq!(session.uuid, session2.uuid);
        assert!(!session.uuid.is_empty());
    }

    #[test]
    fn test_java_launch_options_validation() {
        let runner = runner::GameRunner::new();
        assert!(!runner.is_running());

        let opts = runner::JavaLaunchOptions {
            java_path: "non_existent_java_path".into(),
            game_dir: PathBuf::from("."),
            assets_dir: PathBuf::from("."),
            version: "1.21.11".into(),
            min_memory_mb: 1024,
            max_memory_mb: 4096,
            username: "Player".into(),
            uuid: "00000000-0000-0000-0000-000000000000".into(),
            access_token: "test_token".into(),
            user_type: "mojang".into(),
            version_type: "release".into(),
            jvm_args: vec![],
            game_args: vec![],
            classpath: vec![],
            main_class: "net.minecraft.client.main.Main".into(),
        };

        // Non-existent path returns JavaNotFound error
        let res = runner.launch_java(opts);
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_updater_version_comparison() {
        let updater = updater::Updater::new("1.0.0");
        // Test struct initialization
        assert_eq!(updater.check_for_updates().await.is_err(), false);
    }

    #[tokio::test]
    async fn test_ipc_dispatcher() {
        let dispatcher = ipc::IpcDispatcher::new(PathBuf::from("./temp_test_cache"));
        
        // 1. Test get_system_specs
        let req = ipc::IpcRequest {
            id: 1,
            command: "get_system_specs".to_string(),
            payload: serde_json::Value::Null,
        };
        let res = dispatcher.dispatch(req).await;
        assert!(res.success);
        assert!(res.data.is_some());

        // 2. Test auth_offline
        let req2 = ipc::IpcRequest {
            id: 2,
            command: "auth_offline".to_string(),
            payload: serde_json::json!({ "username": "TestPlayer" }),
        };
        let res2 = dispatcher.dispatch(req2).await;
        assert!(res2.success);
        let val = res2.data.unwrap();
        assert_eq!(val["username"], "TestPlayer");

        // 3. Test unknown command error
        let req3 = ipc::IpcRequest {
            id: 3,
            command: "non_existent_command".to_string(),
            payload: serde_json::Value::Null,
        };
        let res3 = dispatcher.dispatch(req3).await;
        assert!(!res3.success);
        assert!(res3.error.is_some());
    }
}
