use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::lib::KvStore;
mod lib;

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
	
	let mut store = KvStore::open(PathBuf::from("kv.db")).expect("Unable to open kv.db");
	
	match cli.command{
		Commands::Set{key,value}=>{
			store.set(key,value).expect("Unable to set value");
		}
		Commands::Get{key}=>{
			match store.get(key){
				Some(value) => println!("{}",value),
				None => println!("key not found")
			}
		}
		Commands::Rm{ key }=>{
			store.remove(key).expect("unable to remove value");
		}
	}
}
