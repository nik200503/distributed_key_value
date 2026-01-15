use clap::{Parser, Subcommand};
use std::net::TcpStream;
use rust_kv::{Request, Response};
use serde_json::de::{Deserializer};
use serde::{Serialize, Deserialize};
use std::process::exit;

#[derive(Parser)]
#[command(name= "kvs-client")]
struct Cli{
	#[arg(long, default_value = "127.0.0.1:4000")]
	addr: String,
	
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands{
	Set {key: String, value: String},
	Get { key: String},
	Rm { key: String},
	Compact,
}




fn main(){
	let cli = Cli::parse();

	let request = match cli.command{
		Commands::Set {key, value} => Request::Set{key, value},
		Commands::Get {key} => Request::Get {key},
		Commands::Rm {key} => Request::Remove{key},
		Commands::Compact => Request::Compact,
	};
	
	let stream = TcpStream::connect(&cli.addr).unwrap_or_else(|_| {
		eprintln!("Couldn't connect to server at {}. is it running?", cli.addr);
		exit(1);
	});
	
	serde_json::to_writer(&stream, &request).unwrap();
	
	let mut stream_de= Deserializer::from_reader(&stream);
	
	match Response::deserialize(&mut stream_de) {
		Ok(response) => match response {
		Response::Ok(Some(val)) => println!("{}", val),
		Response::Ok(None) => {},
		Response::Err(e) => {
			eprintln!("Errpr: {}",e);
			exit(1);
		},
	},
	Err(_) => {
		eprintln!("Failed to parse response from server.");
		exit(1);
	}
	}
}
