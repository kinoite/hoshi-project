// hoshi/webfetch/src/lib.rs
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::stream::StreamExt; // <-- CHANGED THIS LINE
use std::path::{Path, PathBuf};
use std::error::Error;
use tokio::sync::mpsc;
// REMOVED: use std::io::Result as IoResult;
// REMOVED: use std::time::Instant;

#[derive(Debug)]
pub struct DownloadProgress {
    pub current_bytes: u64,
    pub total_bytes: Option<u64>,
    pub done: bool,
}

pub async fn fetch_url_to_string(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    let response = client.get(url).send().await?.error_for_status()?;
    let text = response.text().await?;
    Ok(text)
}

pub fn get_temp_download_dir() -> PathBuf {
    std::env::temp_dir().join("hoshi_downloads_temp")
}

pub async fn download_file_with_progress(
    url: &str,
    target_dir: &Path,
    file_name: &str,
    tx: mpsc::Sender<DownloadProgress>,
) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    let response = client.get(url).send().await?.error_for_status()?;

    let total_size = response.content_length();
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    tokio::fs::create_dir_all(target_dir).await?;
    let file_path = target_dir.join(file_name);
    let mut file = File::create(&file_path).await?;

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        tx.send(DownloadProgress {
            current_bytes: downloaded,
            total_bytes: total_size,
            done: false,
        }).await?;
    }

    tx.send(DownloadProgress {
        current_bytes: downloaded,
        total_bytes: total_size,
        done: true,
    }).await?;

    Ok(file_path)
}
