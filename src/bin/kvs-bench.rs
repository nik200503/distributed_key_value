use std::net::TcpStream;
use std::time::Instant;
use rust_kv::{Request, Response};
use serde_json::de::Deserializer;
use serde::{Serialize, Deserialize};

fn main(){
	println!("Starting Benchmark...");
	
	let target_addr = "127.0.0.1:4000";
	let iterations = 1000;
	
	let stream = TcpStream::connect(target_addr).expect("Failed to connect to server");
	
	let mut reader = stream.try_clone().expect("Failed to clone stream");
	let mut writer = stream;
	
	let start = Instant::now();
	
	let mut deserializer = Deserializer::from_reader(&mut reader);
	
	for i in 0..iterations{
		let key = format!("key_{}",i);
		let value = format!("value_{}",i);
		
		let request = Request::Set{key, value};
		
		if let Err(e) = serde_json::to_writer(&mut writer, &request){
			eprintln!("Failed to send request: {}",e);
			break;
		}
		
		if let Ok(resp) = Response::deserialize(&mut deserializer){
			match resp{
				Response::Err(e) => eprintln!("Server Error: {}", e),
				_ => {}
			}
		}
	}
	let duration = start.elapsed();
	
	let seconds = duration.as_secs_f64();
	let ops_per_sec = iterations as f64 / seconds;
	
	println!("--- Benchmark results ---");
	println!("Total Requests: {}", iterations);
	println!("Total Time:     {:.4} seconds", seconds);
	println!("Throughput:     {:.2} OPS (Operations Per Second)", ops_per_sec);
	
}
