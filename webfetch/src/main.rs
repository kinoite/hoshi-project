// hoshi/webfetch/src/main.rs
use std::env;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::task::JoinError; 

use webfetch::{download_file_with_progress, DownloadProgress};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <URL> <FILENAME> [TARGET_DIR]", args[0]);
        eprintln!("Example: {} http://example.com/file.zip my_file.zip /tmp", args[0]);
        return Ok(());
    }

   
    let url = args[1].clone();
    let file_name = args[2].clone();
    let target_dir = if args.len() > 3 {
        PathBuf::from(&args[3])
    } else {
        webfetch::get_temp_download_dir()
    };

    println!("Attempting to download: {} to {}/{}", url, target_dir.display(), file_name);

    let (tx, mut rx) = mpsc::channel::<DownloadProgress>(100);

    let target_dir_for_task = target_dir.clone(); // Clone target_dir for the task
    let url_for_task = url.clone(); // Clone url for the task
    let file_name_for_task = file_name.clone(); // Clone file_name for the task

    let download_task = tokio::spawn(async move {
        download_file_with_progress(&url_for_task, &target_dir_for_task, &file_name_for_task, tx).await
    });

    while let Some(progress) = rx.recv().await {
        if let Some(total) = progress.total_bytes {
            print!("\rDownloading: {} / {} bytes ({:.2}%)",
                   progress.current_bytes,
                   total,
                   (progress.current_bytes as f64 / total as f64) * 100.0);
        } else {
            print!("\rDownloading: {} bytes", progress.current_bytes);
        }
        if progress.done {
            println!("\nDownload complete!");
            break;
        }
    }

    match download_task.await? {
        Ok(path) => {
            println!("File saved to: {}", path.display());
            Ok(())
        },
        Err(e) => {
            eprintln!("Error during download: {}", e);
            Err(e)
        }
    }
}
