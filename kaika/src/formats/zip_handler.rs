use std::io::Result;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use zip::write::{FileOptions, ZipWriter};
use zip::read::ZipArchive;

pub async fn create_zip_archive(archive_path: &Path, paths: &[PathBuf]) -> Result<()> {
    let file_std = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(archive_path)
        .await?
        .into_std()
        .await;

    let mut zip = ZipWriter::new(file_std);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    for path in paths {
        let metadata = tokio::fs::metadata(path).await?;
        let filename = path.file_name().unwrap_or_default().to_str().unwrap_or("").to_string();

        if metadata.is_file() {
            zip.start_file(&filename, options)?;
            let mut file_to_archive = File::open(path).await?.into_std().await;
            std::io::copy(&mut file_to_archive, &mut zip)?;
        } else if metadata.is_dir() {
            zip.add_directory(&format!("{}/", filename), options)?;
        } else {
            eprintln!("Warning: Skipping unsupported file type for zip: {}", path.display());
        }
    }

    zip.finish()?;
    Ok(())
}

pub async fn extract_zip_archive(archive_path: &Path, output_dir: &Path) -> Result<()> {
    tokio::fs::create_dir_all(output_dir).await?;

    let file_std = File::open(archive_path)
        .await?
        .into_std()
        .await;

    let mut zip_archive = ZipArchive::new(file_std)?;

    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i)?;
        let outpath = output_dir.join(file.name());

        if file.name().ends_with('/') {
            tokio::fs::create_dir_all(&outpath).await?;
        } else {
            if let Some(parent) = outpath.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            let mut outfile = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&outpath)
                .await?
                .into_std()
                .await;

            std::io::copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                tokio::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).await?;
            }
        }
    }

    Ok(())
}
