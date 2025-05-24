use crate::registry::PackageRegistry;

pub async fn handle(package_name: &str) {
    let registry_path = PackageRegistry::get_install_path();
    let mut registry = PackageRegistry::load(&registry_path).await;

    match registry.remove(package_name, None) {
        Some(pkg) => {
            println!("Successfully removed package: {} v{} from registry.", pkg.name, pkg.version);
            registry.save(&registry_path).await;
            println!("Package registry updated.");
        },
        None => {
            println!("Package '{}' not found in registry.", package_name);
        }
    }
}
