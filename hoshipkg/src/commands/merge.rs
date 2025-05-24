use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use indicatif::{ProgressBar, ProgressStyle};
use dialoguer::Confirm;
use tokio::sync::mpsc;
use tokio::task;

use webfetch;
use kaika;
use crate::registry::{PackageRegistry, InstalledPackage};

use webfetch::DownloadProgress;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub download_url: String,
    pub size_mb: u32,
    pub archive_type: String,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConstellationMetadata {
    pub name: String,
    pub description: String,
    pub packages: Vec<PackageMetadata>,
}

#[derive(Debug)]
pub struct Constellation {
    pub name: String,
    pub metadata_url: String,
}

impl Constellation {
    pub fn default_constellations() -> Vec<Self> {
        vec![
            Constellation {
                name: "Hoshi Core".to_string(),
                metadata_url: "http://localhost:8000/hoshi-core-constellation.json".to_string(),
            },
        ]
    }
}

pub async fn handle(package_name: &str) {
    println!("\nStarting constellation sync...");

    let constellations = Constellation::default_constellations();
    let mut all_available_packages: Vec<PackageMetadata> = Vec::new();

    let pb = ProgressBar::new(constellations.len() as u64);
    let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
        .unwrap();
    pb.set_style(style);

    for (i, constellation) in constellations.iter().enumerate() {
        pb.set_message(format!("Syncing: {}", constellation.name));
        pb.set_position(i as u64);

        match webfetch::fetch_url_to_string(&constellation.metadata_url).await {
            Ok(metadata_content) => {
                match serde_json::from_str::<ConstellationMetadata>(&metadata_content) {
                    Ok(mut meta) => {
                        println!("Successfully synced constellation: {}", meta.name);
                        all_available_packages.append(&mut meta.packages);
                    },
                    Err(e) => {
                        eprintln!("Error parsing metadata for {}: {}", constellation.name, e);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error fetching metadata for {}: {}", constellation.name, e);
            }
        }
    }
    pb.finish_with_message("Constellation sync complete.");

    println!("\nResolving dependencies...");
    let mut packages_to_merge: Vec<PackageMetadata> = Vec::new();
    let target_package = all_available_packages.iter().find(|p| p.name == *package_name);

    if let Some(pkg) = target_package {
        packages_to_merge.push(pkg.clone());

        if let Some(deps) = &pkg.dependencies {
            for dep_name in deps {
                if let Some(dep_pkg) = all_available_packages.iter().find(|p| p.name == *dep_name) {
                    if !packages_to_merge.iter().any(|p| p.name == dep_pkg.name) {
                        packages_to_merge.push(dep_pkg.clone());
                    }
                } else {
                    eprintln!("Warning: Dependency '{}' for '{}' not found in any constellation.", dep_name, pkg.name);
                }
            }
        }
    } else {
        panic!("Package '{}' not found in any constellation.", package_name);
    }

    if packages_to_merge.is_empty() {
        println!("No packages to merge.");
        return;
    }

    println!("\nPackages to merge:");
    for pkg in &packages_to_merge {
        println!(" - {} v{} ({} MB)", pkg.name, pkg.version, pkg.size_mb);
    }

    let confirmation = Confirm::new()
        .with_prompt("Do you want to merge the listed packages?")
        .interact()
        .unwrap();

    if !confirmation {
        println!("Merge aborted by user.");
        return;
    }

    println!("\nStarting package downloads...");
    let temp_download_dir = webfetch::get_temp_download_dir();
    tokio::fs::create_dir_all(&temp_download_dir).await.unwrap();

    let mut download_tasks = Vec::new();
    for pkg in &packages_to_merge {
        let pkg_name_outer = pkg.name.clone();
        let pkg_version_outer = pkg.version.clone();
        let file_name_outer: String;

        let download_url = pkg.download_url.clone();
        let download_target_dir = temp_download_dir.clone();

        file_name_outer = download_url.rsplit_once('/').map_or(
            format!("{}-{}.archive", pkg_name_outer, pkg_version_outer),
            |(_, name)| name.to_string()
        );

        let pb = ProgressBar::new(100);
        let style = ProgressStyle::with_template(
            "{msg} {spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes_per_sec} {bytes}/{total_bytes} ({eta})"
        ).unwrap();
        pb.set_style(style);
        pb.set_message(format!("{} v{}", pkg_name_outer, pkg_version_outer));
        pb.set_length(0);

        let (tx, mut rx) = mpsc::channel::<DownloadProgress>(100);

        let url_for_download_task = download_url.clone();
        let target_dir_for_download_task = download_target_dir.clone();
        let file_name_for_download_task = file_name_outer.clone();

        let download_handle = task::spawn(async move {
            webfetch::download_file_with_progress(
                &url_for_download_task,
                &target_dir_for_download_task,
                &file_name_for_download_task,
                tx,
            ).await.unwrap()
        });

        let pkg_name_for_pb_task = pkg_name_outer.clone();
        let pkg_version_for_pb_task = pkg_version_outer.clone();

        let pb_handle = task::spawn(async move {
            let mut last_bytes = 0;
            let mut last_time = std::time::Instant::now();
            while let Some(progress) = rx.recv().await {
                if let Some(total) = progress.total_bytes {
                    if pb.length() == Some(0) || pb.length().is_none() {
                        pb.set_length(total);
                    }
                }
                pb.set_position(progress.current_bytes);

                let now = std::time::Instant::now();
                let elapsed_secs = (now - last_time).as_secs_f64();
                if elapsed_secs >= 0.1 {
                    let bytes_since_last_update = progress.current_bytes.saturating_sub(last_bytes);
                    let speed = (bytes_since_last_update as f64 / elapsed_secs) / (1024.0 * 1024.0);
                    pb.set_message(format!("{} v{} ({:.2} MB/s)", pkg_name_for_pb_task, pkg_version_for_pb_task, speed));
                    last_bytes = progress.current_bytes;
                    last_time = now;
                }

                if progress.done {
                    pb.finish_with_message(format!("{} v{} downloaded.", pkg_name_for_pb_task, pkg_version_for_pb_task));
                    break;
                }
            }
            pb.finish();
        });

        download_tasks.push((download_handle, pb_handle, pkg_name_outer, pkg_version_outer, file_name_outer));
    }

    let mut downloaded_package_paths: Vec<(String, String, PathBuf)> = Vec::new();
    for (download_handle, pb_handle, pkg_name, pkg_version, _file_name) in download_tasks {
        let downloaded_file_path = download_handle.await.unwrap();
        pb_handle.await.unwrap();

        downloaded_package_paths.push((pkg_name, pkg_version, downloaded_file_path));
    }
    println!("All packages downloaded. Shutting down webfetch...");

    println!("\nStarting package extraction...");
    let install_base_dir = PathBuf::from("./hoshi_packages");
    tokio::fs::create_dir_all(&install_base_dir).await.unwrap();

    let registry_path = PackageRegistry::get_install_path();
    let mut registry = PackageRegistry::load(&registry_path).await;

    for (pkg_name, pkg_version, downloaded_file_path) in downloaded_package_paths {
        let _pkg_metadata = packages_to_merge.iter().find(|p| p.name == pkg_name && p.version == pkg_version)
            .unwrap();

        let package_install_dir = install_base_dir.join(&pkg_name).join(&pkg_version);
        println!("Extracting {} to {}...", pkg_name, package_install_dir.display());

        kaika::extract_archive(&downloaded_file_path, &package_install_dir).await;

        println!("Extracted: {}", pkg_name);

        registry.add(InstalledPackage {
            name: pkg_name.clone(),
            version: pkg_version.clone(),
            install_path: package_install_dir.clone(),
        });
    }
    println!("All packages extracted. Powering down kaika...");

    registry.save(&registry_path).await;
    println!("Package registry updated.");

    println!("\nMerge complete!");
}
