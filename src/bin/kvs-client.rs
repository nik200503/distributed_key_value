use clap::{Parser, Subcommand};
use rust_kv::{Request, Response};
use serde::Deserialize;
use serde_json::de::Deserializer;
use std::net::TcpStream;
use std::process::exit;

#[derive(Parser)]
#[command(name = "kvs-client")]
struct Cli {
    #[arg(long, default_value = "127.0.0.1:4000")]
    addr: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
    Compact,
    Scan {
        start: String,
        end: String
    },
}

fn main() {
    let cli = Cli::parse();

    let is_get = matches!(cli.command, Commands::Get { .. });

    let request = match cli.command {
        Commands::Set { key, value } => Request::Set { key, value },
        Commands::Get { key } => Request::Get { key },
        Commands::Rm { key } => Request::Remove { key },
        Commands::Compact => Request::Compact,
        Commands::Scan { start, end } => Request::Scan { start, end},
    };

    let stream = TcpStream::connect(&cli.addr).unwrap_or_else(|_| {
        eprintln!("Couldn't connect to server at {}. is it running?", cli.addr);
        exit(1);
    });

    serde_json::to_writer(&stream, &request).unwrap();

    let mut stream_de = Deserializer::from_reader(&stream);

    match Response::deserialize(&mut stream_de) {
        Ok(response) => match response {
            Response::ScanResult(pairs) => {
                println!("Found {} keys:", pairs.len());
                for (k, v) in pairs {
                    println!("{} = {}", k, v);
                }
            }
            Response::Ok(Some(val)) => println!("{}", val),
            Response::Ok(None) => {
                if is_get {
                    println!("key not found");
                }
            }
            Response::Err(e) => {
                eprintln!("Errpr: {}", e);
                exit(1);
            }
        },
        Err(_) => {
            eprintln!("Failed to parse response from server.");
            exit(1);
        }
    }
}
