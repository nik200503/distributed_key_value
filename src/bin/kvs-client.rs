use std::io::prelude::*;
use std::net::TcpStream;

fn main(){
	match TcpStream::connect("127.0.0.1:4000") {
		Ok(mut stream) => {
			println!("Successfully connected to server at port 4000");
			
			let msg = "Hello from the client!";
			stream.write(msg.as_bytes()).unwrap();
			println!("Sent: {}", msg);
			
			let mut buffer = [0; 512];
			match stream.read(&mut buffer){
				Ok(bytes_read) => {
					let response = String::from_utf8_lossy(&buffer[..bytes_read]);
					println!("Server replied: {}", response);
				}
				Err(e) => println!("Failed to read response: {}",e),
			}
		},
		Err(e) => {
			println!("Failed to connect: {}",e);
		}
	}
}
