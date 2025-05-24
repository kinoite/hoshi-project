use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use webfetch; // Use webfetch to fetch constellation metadata

// Re-use structs from hoshipkg for package and constellation metadata
// This assumes hoshipkg's structs are public (which they are) and compatible.
// For a standalone tool, you might want a shared 'hoshi-core-types' crate
// or define them directly here. For now, we'll define them here to avoid circular deps.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub size_mb: f64,
    pub download_url: String,
    pub archive_type: String,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConstellationMetadata {
    pub name: String,
    #[serde(default)]
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
                name: "Hoshi-Core".to_string(),
                metadata_url: "http://localhost:8000/hoshi-core-constellation.json".to_string(),
            },
            Constellation {
                name: "Hoshi-Extra".to_string(),
                metadata_url: "http://localhost:8000/hoshi-extra-constellation.json".to_string(),
            },
        ]
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Search for Hoshi packages", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Search {
        #[arg(short, long)]
        constellation: Option<String>,

        #[arg(required = true)]
        query: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Search { constellation, query } => {
            handle_search_command(constellation.as_deref(), query).await?;
        }
    }

    Ok(())
}

async fn handle_search_command(
    constellation_name: Option<&str>,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let constellations = Constellation::default_constellations();
    let mut all_packages: Vec<PackageMetadata> = Vec::new();

    let target_constellations = if let Some(name) = constellation_name {
        constellations.into_iter().filter(|c| c.name.eq_ignore_ascii_case(name)).collect::<Vec<_>>()
    } else {
        constellations
    };

    if target_constellations.is_empty() {
        println!("No constellations found or specified constellation '{}' does not exist.", constellation_name.unwrap_or_default());
        return Ok(());
    }

    println!("Searching for '{}' in selected constellations...", query);

    for constellation in target_constellations {
        match webfetch::fetch_url_to_string(&constellation.metadata_url).await {
            Ok(metadata_content) => {
                match serde_json::from_str::<ConstellationMetadata>(&metadata_content) {
                    Ok(meta) => {
                        all_packages.extend(meta.packages);
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

    let mut found_packages: Vec<&PackageMetadata> = all_packages.iter()
        .filter(|p| p.name.contains(query) || p.name.eq_ignore_ascii_case(query))
        .collect();

    found_packages.sort_by(|a, b| a.name.cmp(&b.name));

    if found_packages.is_empty() {
        println!("No packages found matching '{}'.", query);
    } else {
        println!("\nFound packages:");
        for pkg in found_packages {
            println!("  {} v{} ({} MB)", pkg.name, pkg.version, pkg.size_mb);
        }
    }

    Ok(())
}
