use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid configuration: {0}")]
    Config(String),
    #[error("Java executable not found at: {0}")]
    JavaNotFound(String),
    #[error("Game launch failed: {0}")]
    LaunchFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaLaunchOptions {
    pub java_path: String,
    pub game_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub version: String,
    pub min_memory_mb: u32,
    pub max_memory_mb: u32,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: String,
    pub version_type: String,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub classpath: Vec<PathBuf>,
    pub main_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockLaunchOptions {
    pub game_dir: Option<PathBuf>,
    pub server_address: Option<String>,
    pub server_port: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEdition {
    Java,
    Bedrock,
}

pub struct GameRunner {
    is_game_running: Arc<AtomicBool>,
}

impl GameRunner {
    pub fn new() -> Self {
        Self {
            is_game_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_game_running.load(Ordering::SeqCst)
    }

    /// Spawns Minecraft Java Edition with high-performance JVM arguments and tuned GC
    pub fn launch_java(&self, opts: JavaLaunchOptions) -> Result<u32, RunnerError> {
        let java_exe = Path::new(&opts.java_path);
        if !java_exe.exists() && opts.java_path != "java" {
            return Err(RunnerError::JavaNotFound(opts.java_path));
        }

        let mut cmd = Command::new(&opts.java_path);

        // Working directory
        cmd.current_dir(&opts.game_dir);

        // Memory configuration
        cmd.arg(format!("-Xms{}M", opts.min_memory_mb.max(256)));
        cmd.arg(format!("-Xmx{}M", opts.max_memory_mb.max(1024)));

        // Enterprise G1GC Low-Latency Tuning
        cmd.arg("-XX:+UseG1GC");
        cmd.arg("-XX:+UnlockExperimentalVMOptions");
        cmd.arg("-XX:G1NewSizePercent=20");
        cmd.arg("-XX:G1ReservePercent=20");
        cmd.arg("-XX:MaxGCPauseMillis=50");
        cmd.arg("-XX:G1HeapRegionSize=32M");

        // Custom JVM args
        for arg in &opts.jvm_args {
            cmd.arg(arg);
        }

        // Natives library path
        let natives_dir = opts.game_dir.join("natives");
        if natives_dir.exists() {
            cmd.arg(format!("-Djava.library.path={}", natives_dir.display()));
        }

        // Classpath construction
        let sep = if cfg!(windows) { ";" } else { ":" };
        let cp_joined = opts
            .classpath
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(sep);

        if !cp_joined.is_empty() {
            cmd.arg("-cp").arg(cp_joined);
        }

        // Main class
        cmd.arg(&opts.main_class);

        // Standard Minecraft game arguments
        cmd.arg("--username").arg(&opts.username);
        cmd.arg("--version").arg(&opts.version);
        cmd.arg("--gameDir").arg(&opts.game_dir);
        cmd.arg("--assetsDir").arg(&opts.assets_dir);
        cmd.arg("--uuid").arg(&opts.uuid);
        cmd.arg("--accessToken").arg(&opts.access_token);
        cmd.arg("--userType").arg(&opts.user_type);
        cmd.arg("--versionType").arg(&opts.version_type);

        for arg in &opts.game_args {
            cmd.arg(arg);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        log::info!("Spawning Java Edition process for user: {}", opts.username);
        let child = cmd.spawn().map_err(RunnerError::Io)?;
        let pid = child.id();

        let running_flag = self.is_game_running.clone();
        running_flag.store(true, Ordering::SeqCst);

        // Background monitor thread
        std::thread::spawn(move || {
            let mut child = child;
            let status = child.wait();
            running_flag.store(false, Ordering::SeqCst);
            log::info!("Minecraft process (PID {}) terminated with status: {:?}", pid, status);
        });

        Ok(pid)
    }

    /// Launches Minecraft Bedrock Edition natively across OS platforms
    pub fn launch_bedrock(&self, opts: BedrockLaunchOptions) -> Result<u32, RunnerError> {
        log::info!("Initiating Bedrock Edition launch sequence");

        #[cfg(target_os = "windows")]
        {
            // Windows 10/11: Native UWP Protocol activation or AppExecutionAlias
            let uri = if let (Some(addr), Some(port)) = (opts.server_address, opts.server_port) {
                format!("minecraft://connect?serverUrl={}&serverPort={}", addr, port)
            } else {
                "minecraft:".to_string()
            };

            let mut cmd = Command::new("cmd");
            cmd.args(["/c", "start", "", &uri]);
            let child = cmd.spawn().map_err(RunnerError::Io)?;
            let pid = child.id();

            self.is_game_running.store(true, Ordering::SeqCst);
            let running_flag = self.is_game_running.clone();

            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(5));
                running_flag.store(false, Ordering::SeqCst);
            });

            return Ok(pid);
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: mcpelauncher-client / flatpak
            let launcher_cmd = if Path::new("/usr/bin/mcpelauncher-client").exists() {
                "mcpelauncher-client"
            } else {
                "flatpak"
            };

            let mut cmd = Command::new(launcher_cmd);
            if launcher_cmd == "flatpak" {
                cmd.args(["run", "io.mrarm.mcpelauncher"]);
            }

            let child = cmd.spawn().map_err(RunnerError::Io)?;
            let pid = child.id();
            self.is_game_running.store(true, Ordering::SeqCst);
            return Ok(pid);
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: Native open invocation for Bedrock wrapper
            let mut cmd = Command::new("open");
            cmd.args(["-a", "mcpelauncher-ui-qt"]);
            let child = cmd.spawn().map_err(RunnerError::Io)?;
            return Ok(child.id());
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Err(RunnerError::LaunchFailed("Bedrock is unsupported on this platform".into()))
        }
    }
}
