use clap::{Parser, Subcommand};

mod commands;
mod registry;
use crate::commands::list;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Merge {
        name: String,
    },
    Sync {
        constellation: String,
    },
    List,
    Delete {
        name: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Merge { name } => {
            commands::merge::handle(name).await;
        },
        Commands::Sync { constellation } => {
            commands::sync::handle(constellation).await;
        },
        Commands::List => {
            list::handle().await;
        },
        Commands::Delete { name } => {
            commands::delete::handle(name).await;
        },
    }
}
