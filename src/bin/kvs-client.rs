use std::io::prelude::*;
use std::net::TcpStream;
use rust_kv::{Request, Response};
use serde_json::de::{Deserializer};
use serde::{Serialize, Deserialize};

fn main(){

	let mut stream = TcpStream::connect("127.0.0.1:4000").unwrap();
	
	let request = Request::Set{
		key: "framework".to_string(),
		value: "actix".to_string(),
	};
	
	serde_json::to_writer(&stream, &request).unwrap();
	println!("Sent Request: {:?}", request);
	
	let mut stream_de = Deserializer::from_reader(&stream);
	let response : Response = Response::deserialize(&mut stream_de).unwrap();
	
	println!("Recived response: {:?}", response);
	
	
}
