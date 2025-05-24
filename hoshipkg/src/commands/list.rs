use crate::registry::PackageRegistry;

pub async fn handle() {
    let registry_path = PackageRegistry::get_install_path();
    let registry = PackageRegistry::load(&registry_path).await;

    let packages = registry.list_packages();
    if packages.is_empty() {
        println!("No packages installed.");
    } else {
        println!("Installed packages:");
        for pkg in packages {
            println!(" - {} v{} installed at {}", pkg.name, pkg.version, pkg.install_path.display());
        }
    }
}
