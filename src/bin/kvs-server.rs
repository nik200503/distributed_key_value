use std::net::{TcpListener, TcpStream};
use rust_kv::{KvStore, Request, Response};
use serde_json::de::Deserializer;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;


fn main(){
	let listener = TcpListener::bind("127.0.0.1:4000").expect("Failed to bind");
	
	println!("Server is listening on port 4000");
	
	let mut store = KvStore::open(PathBuf::from("kv.db")).expect("failed to bind");
	
	let shared_store = Arc::new(Mutex::new(store));
	
	for stream in listener.incoming(){
		match stream{
			Ok(stream)=>{
			
				let store_handle = shared_store.clone();
				println!("New connection! Spawning thread...");
				thread::spawn(move || {
					handle_connection(stream, store_handle);
				});
			}
			Err(e) => println!("connection Failed : {}",e),
		}
	}
}

fn handle_connection(stream: TcpStream, db: Arc<Mutex<KvStore>>){
	let mut stream_de = Deserializer::from_reader(&stream);
	
	while let Ok(request) = Request::deserialize(&mut stream_de){
		println!("Processing : {:?}", request);
		let mut store = db.lock().unwrap();
		
		let response = match request{
			Request::Set {key, value} => {
				match store.set(key, value){
					Ok(_) => Response::Ok(None),
					Err(e) => Response::Err(e.to_string()),
				}
			},	
			Request::Get {key} =>{
				match store.get(key){
					Some(val) => Response::Ok(Some(val)),
					None => Response::Ok(None),
				}
			},
			Request::Remove{ key }=>{
				match store.remove(key){
					Ok(_) => Response::Ok(None),
					Err(e) => Response::Err(e.to_string()),
				}
			},
			Request::Compact =>{
				println!("Compacting database...");
				match store.compact(){
					Ok(_) => Response::Ok(None),
					Err(e) => Response::Err(e.to_string()),
				}
			},
		};
		
		if serde_json::to_writer(&stream, &response).is_err(){
			break;
		};
	}
}
