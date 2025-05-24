use serde::{Deserialize, Serialize};
use tokio::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub install_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PackageRegistry {
    packages: HashMap<String, InstalledPackage>,
}

impl PackageRegistry {
    pub fn get_install_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("hoshi")
            .join("registry.json")
    }

    pub async fn load(path: &Path) -> Self {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await.expect("Failed to create parent directory for registry");
            }
            return PackageRegistry::default();
        }

        let content = fs::read_to_string(path).await.expect("Failed to read registry file");
        serde_json::from_str(&content).expect("Failed to parse registry content")
    }

    pub async fn save(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.expect("Failed to create parent directory for registry save");
        }

        let content = serde_json::to_string_pretty(&self).expect("Failed to serialize registry");
        fs::write(path, content).await.expect("Failed to write registry file");
    }

    pub fn add(&mut self, package: InstalledPackage) {
        let key = format!("{}-{}", package.name, package.version);
        self.packages.insert(key, package);
    }

    pub fn remove(&mut self, name: &str, version: Option<&str>) -> Option<InstalledPackage> {
        let key_to_remove = match version {
            Some(v) => format!("{}-{}", name, v),
            None => {
                if let Some((key, _)) = self.packages.iter().find(|(_, pkg)| pkg.name == name) {
                    key.clone()
                } else {
                    return None;
                }
            }
        };
        self.packages.remove(&key_to_remove)
    }

    pub fn list_packages(&self) -> Vec<&InstalledPackage> {
        self.packages.values().collect()
    }
}
