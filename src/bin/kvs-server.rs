use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn main(){
	let listener = TcpListener::bind("127.0.0.1:4000").expect("failed to bind to address");
	
	println!("Server is listening on 127.0.0.1:4000");
	
	for stream in listener.incoming(){
		match stream{
			Ok(stream)=>{
				println!("New connection established!");
				handle_connection(stream);
			}
			Err(e)=>{
				println!("Failed to establish connection : {}",e);
			}
		}
	}
}

fn handle_connection(mut stream: TcpStream){
	let mut buffer = [0; 512];
	
	match stream.read(&mut buffer){
		Ok(bytes_read) => {
			if bytes_read == 0{
				return;
			}
			
			let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
			println!("Recived: {}", received_data);
			
			let response = "Message received";
			stream.write(response.as_bytes()).unwrap();
		}
		Err(e) => println!("Failed to read from connection: {}", e),
	}
}
