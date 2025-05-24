use std::io::{self, Result};
use std::path::{Path, PathBuf};

mod formats;

pub async fn create_archive(archive_path: &Path, paths: &[PathBuf]) -> Result<()> {
    let ext = archive_path.extension().and_then(|s| s.to_str());

    match ext {
        Some("tar") => formats::tar_handler::create_tar_archive(archive_path, paths).await,
        Some("gz") => formats::tar_handler::create_tar_gz_archive(archive_path, paths).await,
        Some("bz2") => formats::tar_handler::create_tar_bz2_archive(archive_path, paths).await,
        Some("xz") => formats::tar_handler::create_tar_xz_archive(archive_path, paths).await,
        Some("zip") => formats::zip_handler::create_zip_archive(archive_path, paths).await,
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Unsupported archive format: {}", archive_path.display()),
        )),
    }
}

pub async fn extract_archive(archive_path: &Path, output_dir: &Path) -> Result<()> {
    let ext = archive_path.extension().and_then(|s| s.to_str());

    match ext {
        Some("tar") => formats::tar_handler::extract_tar_archive(archive_path, output_dir).await,
        Some("gz") => formats::tar_handler::extract_tar_gz_archive(archive_path, output_dir).await,
        Some("bz2") => formats::tar_handler::extract_tar_bz2_archive(archive_path, output_dir).await,
        Some("xz") => formats::tar_handler::extract_tar_xz_archive(archive_path, output_dir).await,
        Some("zip") => formats::zip_handler::extract_zip_archive(archive_path, output_dir).await,
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Unsupported archive format: {}", archive_path.display()),
        )),
    }
}
