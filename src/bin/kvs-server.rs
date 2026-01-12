use std::net::{TcpListener, TcpStream};
use rust_kv::{Request, Response};
use serde_json::de::Deserializer;
use serde::{Serialize, Deserialize};


fn main(){
	let listener = TcpListener::bind("127.0.0.1:4000").unwrap();
	
	println!("Server is listening on port 4000");
	
	for stream in listener.incoming(){
		match stream{
			Ok(stream)=>{
				handle_connection(stream);
			}
			Err(e) => println!("connection Failed : {}",e),
		}
	}
}

fn handle_connection(stream: TcpStream){
	let mut stream_de = Deserializer::from_reader(&stream);
	
	if let Ok(request) = Request::deserialize(&mut stream_de){
		println!("Recived request : {:?}", request);
		
		let response = match request{
			Request::Set {key, value} => {
				println!("user wants to SET {} to GET {}", key, value);
				Response::Ok(Some("mock_value".to_string()))
			},
			Request::Get {key} =>{
				println!("user wants to GET {}", key);
				Response::Err("Key not found".to_string())
			},
			Request::Remove{ key }=>{
				println!("User wants to REMOVE {}", key);
				Response::Err("Key not found".to_string())
			},
		};
		
		serde_json::to_writer(&stream, &response).unwrap();
	}
}
