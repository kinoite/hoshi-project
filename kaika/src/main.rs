use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "A powerful, feature-rich, and fast alternative to tar.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Create {
        #[arg(short, long)]
        archive: PathBuf,

        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
    Extract {
        #[arg(short, long)]
        archive: PathBuf,

        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Create { archive, paths } => {
            println!("Creating archive: {} from {:?}", archive.display(), paths);
            kaika::create_archive(archive, paths).await?;
            println!("Archive created successfully!");
        }
        Commands::Extract { archive, output_dir } => {
            // Create a longer-lived PathBuf for the default output directory
            let default_output_dir = PathBuf::from(".");
            let output = output_dir.as_deref().unwrap_or(&default_output_dir);
            println!("Extracting archive: {} to {}", archive.display(), output.display());
            kaika::extract_archive(archive, output).await?;
            println!("Archive extracted successfully!");
        }
    }

    Ok(())
}
