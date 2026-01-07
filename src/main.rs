use clap::{Parser, Subcommand};
use std::process::exit;

#[derive(Parser)]
#[command(name ="rust_kv")]
#[command(about = "A distributed key-value store", long_about = None)]

struct Cli{
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands{
	Set{
		key: String,
		value: String,
	},
	Get{
		key: String,
	},
	Rm{
		key: String,
	},
}

fn main(){
	let cli = Cli::parse();
	
	match &cli.command{
		Commands::Set{key,value}=>{
			println!("Unimplemented: Set {} to {}", key, value);
		}
		Commands::Get{key}=>{
			println!("Unimplemented: Get {}", key);
		}
		Commands::Rm{ key }=>{
		println!("Unimplemented: Rm {}", key);
		}
	}
}
