use webfetch;
use crate::commands::merge::{Constellation, ConstellationMetadata};

pub async fn handle(constellation_name: &str) {
    println!("Attempting to sync constellation: {}", constellation_name);

    let constellations = Constellation::default_constellations();
    let target_constellation = constellations.iter()
        .find(|c| c.name.eq_ignore_ascii_case(constellation_name));

    if let Some(constellation) = target_constellation {
        println!("Syncing: {}", constellation.name);

        match webfetch::fetch_url_to_string(&constellation.metadata_url).await {
            Ok(metadata_content) => {
                match serde_json::from_str::<ConstellationMetadata>(&metadata_content) {
                    Ok(meta) => {
                        println!("\nSuccessfully synced constellation: {}", meta.name);
                        println!("Found {} packages.", meta.packages.len());
                    },
                    Err(e) => {
                        eprintln!("Error parsing metadata for {}: {}", constellation.name, e);
                        panic!("Failed to parse metadata for {}: {}", constellation.name, e);
                    }
                }
            },
            Err(e) => {
                eprintln!("Error fetching metadata for {}: {}", constellation.name, e);
                panic!("Failed to fetch metadata for {}: {}", constellation.name, e);
            }
        }
        println!("Sync complete.");
    } else {
        panic!("Constellation '{}' not found.", constellation_name);
    }
}
