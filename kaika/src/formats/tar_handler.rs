use std::io::{self, Result};
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tar::{Archive, Builder};
use flate2::bufread::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression as Flate2Compression;
use bzip2::bufread::BzDecoder;
use bzip2::write::BzEncoder;
use bzip2::Compression as Bz2Compression;
use xz2::bufread::XzDecoder;
use xz2::write::XzEncoder;
use std::io::{Read, Write, BufReader};

fn get_entry_name(path: &Path) -> Result<PathBuf> {
    std::env::current_dir()
        .map(|cwd| path.strip_prefix(&cwd).unwrap_or(path).to_path_buf())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to get current directory: {}", e)))
}

async fn create_tar_with_compression<W: Write + 'static + Send + Unpin>(
    _archive_path: &Path,
    paths: &[PathBuf],
    writer: W,
) -> Result<()> {
    let mut builder = Builder::new(writer);

    for path in paths {
        let entry_name = get_entry_name(path)?;
        if tokio::fs::metadata(path).await?.is_dir() {
            builder.append_dir_all(&entry_name, path)?;
        } else {
            builder.append_path_with_name(path, &entry_name)?;
        }
    }
    builder.finish()?;
    Ok(())
}

async fn extract_tar_with_decompression<R: Read + 'static + Send + Unpin>(
    _archive_path: &Path,
    output_dir: &Path,
    reader: R,
) -> Result<()> {
    tokio::fs::create_dir_all(output_dir).await?;
    let mut archive = Archive::new(reader);
    archive.unpack(output_dir)?;
    Ok(())
}

pub async fn create_tar_archive(archive_path: &Path, paths: &[PathBuf]) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(archive_path)
        .await?
        .into_std()
        .await;

    create_tar_with_compression(archive_path, paths, file).await
}

pub async fn create_tar_gz_archive(archive_path: &Path, paths: &[PathBuf]) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(archive_path)
        .await?
        .into_std()
        .await;
    let enc = GzEncoder::new(file, Flate2Compression::default());
    create_tar_with_compression(archive_path, paths, enc).await
}

pub async fn create_tar_bz2_archive(archive_path: &Path, paths: &[PathBuf]) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(archive_path)
        .await?
        .into_std()
        .await;
    let enc = BzEncoder::new(file, Bz2Compression::default());
    create_tar_with_compression(archive_path, paths, enc).await
}

pub async fn create_tar_xz_archive(archive_path: &Path, paths: &[PathBuf]) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(archive_path)
        .await?
        .into_std()
        .await;
    let enc = XzEncoder::new(file, 6); // Changed to use integer directly
    create_tar_with_compression(archive_path, paths, enc).await
}

pub async fn extract_tar_archive(archive_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(archive_path)
        .await?
        .into_std()
        .await;
    extract_tar_with_decompression(archive_path, output_dir, file).await
}

pub async fn extract_tar_gz_archive(archive_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(archive_path)
        .await?
        .into_std()
        .await;
    let dec = GzDecoder::new(BufReader::new(file));
    extract_tar_with_decompression(archive_path, output_dir, dec).await
}

pub async fn extract_tar_bz2_archive(archive_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(archive_path)
        .await?
        .into_std()
        .await;
    let dec = BzDecoder::new(BufReader::new(file));
    extract_tar_with_decompression(archive_path, output_dir, dec).await
}

pub async fn extract_tar_xz_archive(archive_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(archive_path)
        .await?
        .into_std()
        .await;
    let dec = XzDecoder::new(BufReader::new(file));
    extract_tar_with_decompression(archive_path, output_dir, dec).await
}
