use crate::lib::KvStore;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rust_kv::Request;
use std::path::PathBuf;

mod lib;

#[derive(Parser)]
#[command(name = "rust_kv")]
#[command(about = "A distributed key-value store", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
    Compact,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut store =
        KvStore::open(PathBuf::from("kv.db")).context("failed to open the database file")?;

    match cli.command {
        Commands::Set { key, value } => {
            store
                .set(key, value)
                .context("failed to save key-value pair")?;
        }
        Commands::Get { key } => match store.get(key) {
            Some(value) => println!("{}", value),
            None => println!("key not found"),
        },
        Commands::Rm { key } => {
            store.remove(key).context("failed to remove key")?;
        }
        Commands::Compact => {
            store.compact().context("failed to compact database")?;
        }
    }
    Ok(())
}
