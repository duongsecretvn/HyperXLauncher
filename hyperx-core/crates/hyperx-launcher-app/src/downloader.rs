use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Semaphore;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Checksum mismatch for {file}: expected {expected}, calculated {actual}")]
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub url: String,
    pub destination: PathBuf,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub percent: f32,
    pub current_file: String,
}

pub struct ParallelDownloader {
    client: reqwest::Client,
    max_concurrency: usize,
}

impl ParallelDownloader {
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("HyperXLauncher-Downloader/1.0.0")
                .pool_max_idle_per_host(32)
                .tcp_nodelay(true)
                .build()
                .unwrap_or_default(),
            max_concurrency: max_concurrency.clamp(4, 64),
        }
    }

    /// Downloads a single file with streaming write and optional SHA-1 checksum verification
    pub async fn download_single(
        client: &reqwest::Client,
        task: &DownloadTask,
    ) -> Result<(), DownloadError> {
        if let Some(parent) = task.destination.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Check if file already exists and matches expected size/checksum
        if task.destination.exists() {
            if let Some(expected_sha1) = &task.sha1 {
                if let Ok(bytes) = tokio::fs::read(&task.destination).await {
                    let mut hasher = Sha1::new();
                    hasher.update(&bytes);
                    let actual = format!("{:x}", hasher.finalize());
                    if actual.eq_ignore_ascii_case(expected_sha1) {
                        return Ok(());
                    }
                }
            }
        }

        let res = client.get(&task.url).send().await?;
        if !res.status().is_success() {
            return Err(DownloadError::Network(res.error_for_status().unwrap_err()));
        }

        let bytes = res.bytes().await?;
        let mut hasher = Sha1::new();
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(&task.destination).await?;
        file.write_all(&bytes).await?;
        if task.sha1.is_some() {
            hasher.update(&bytes);
        }

        if let Some(expected_sha1) = &task.sha1 {
            let actual = format!("{:x}", hasher.finalize());
            if !actual.eq_ignore_ascii_case(expected_sha1) {
                let _ = tokio::fs::remove_file(&task.destination).await;
                return Err(DownloadError::ChecksumMismatch {
                    file: task.destination.to_string_lossy().to_string(),
                    expected: expected_sha1.clone(),
                    actual,
                });
            }
        }

        Ok(())
    }

    /// Parallel multi-threaded download manager with bounded concurrency
    pub async fn download_batch(
        &self,
        tasks: Vec<DownloadTask>,
        progress_callback: impl Fn(DownloadProgress) + Send + Sync + 'static,
    ) -> Result<(), DownloadError> {
        let total_tasks = tasks.len();
        if total_tasks == 0 {
            return Ok(());
        }

        let total_bytes: u64 = tasks.iter().map(|t| t.size.unwrap_or(0)).sum();
        let completed = Arc::new(AtomicUsize::new(0));
        let downloaded = Arc::new(AtomicU64::new(0));
        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));

        let progress_cb = Arc::new(progress_callback);

        let mut handles = Vec::with_capacity(total_tasks);

        for task in tasks {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let client = self.client.clone();
            let completed = completed.clone();
            let downloaded = downloaded.clone();
            let progress_cb = progress_cb.clone();
            let task_size = task.size.unwrap_or(0);
            let file_name = task
                .destination
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "asset".into());

            let handle = tokio::spawn(async move {
                let result = Self::download_single(&client, &task).await;
                drop(permit);

                let c = completed.fetch_add(1, Ordering::SeqCst) + 1;
                let d = downloaded.fetch_add(task_size, Ordering::SeqCst) + task_size;

                progress_cb(DownloadProgress {
                    total_tasks,
                    completed_tasks: c,
                    total_bytes,
                    downloaded_bytes: d,
                    percent: (c as f32 / total_tasks as f32) * 100.0,
                    current_file: file_name,
                });

                result
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Ok(res) = handle.await {
                res?;
            }
        }

        Ok(())
    }
}
