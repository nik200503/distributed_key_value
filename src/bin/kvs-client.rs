use clap::{Parser, Subcommand};
use std::net::TcpStream;
use rust_kv::{Request, Response};
use serde_json::de::{Deserializer};
use serde::{Serialize, Deserialize};
use std::process::exit;

#[derive(Parser)]
#[command(name= "kvs-client")]
struct Cli{
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
	
	let stream = TcpStream::connect("127.0.0.1:4000").unwrap_or_else(|_| {
		eprintln!("Couldn't connect to server. is it running?");
		exit(1);
	});
	
	serde_json::to_writer(&stream, &request).unwrap();
	
	let mut stream_de= Deserializer::from_reader(&stream);
	let response = Response::deserialize(&mut stream_de).unwrap();
	
	match response {
		Response::Ok(Some(val)) => println!("{}", val),
		Response::Ok(None) => {},
		Response::Err(e) => {
			eprintln!("Errpr: {}",e);
			exit(1);
		},
	}
}
