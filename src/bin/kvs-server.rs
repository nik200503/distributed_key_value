use std::net::{TcpListener, TcpStream};
use rust_kv::{KvStore, Request, Response};
use serde_json::de::Deserializer;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use clap::Parser;
use log::{info, error, LevelFilter};

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct ServerCli{
	#[arg(long , default_value = "127.0.0.1")]
	addr: String,
	
	#[arg(long, default_value = "4000")]
	port: u16,
}

fn main(){

	env_logger::builder()
		.filter_level(LevelFilter::Info)
		.init();

	let args = ServerCli::parse();
	
	let bind_addr = format!("{}:{}", args.addr, args.port);
	info!("kvs-server version {}", env!("CARGO_PKG_VERSION"));
	info!("Listening on {}", bind_addr);
	
	let listener = TcpListener::bind(&bind_addr).expect("Failed to bind");
	
	info!("Opening database at ./kv.db");
	
	let store = KvStore::open(PathBuf::from("kv.db")).expect("failed to open db");
	
	let shared_store = Arc::new(Mutex::new(store));
	
	for stream in listener.incoming(){
		match stream{
			Ok(stream)=>{
			
				let store_handle = shared_store.clone();
				thread::spawn(move || {
					handle_connection(stream, store_handle);
				});
			}
			Err(e) => error!("connection Failed : {}",e),
		}
	}
}

fn handle_connection(stream: TcpStream, db: Arc<Mutex<KvStore>>){
	let peer_addr = stream.peer_addr().unwrap();
	info!("New connection from {}", peer_addr);

	let mut stream_de = Deserializer::from_reader(&stream);
	
	while let Ok(request) = Request::deserialize(&mut stream_de){
		log::debug!("Processing request from {}: {:?}", peer_addr, request);
	
		let mut store = db.lock().unwrap();
		
		let response = match request{
			Request::Set {key, value} => {
				match store.set(key, value){
					Ok(_) => Response::Ok(None),
					Err(e) => {
						error!("Set failed: {}",e);
						Response::Err(e.to_string())
					}
				}
			}	
			Request::Get {key} =>{
				match store.get(key){
					Some(val) => Response::Ok(Some(val)),
					None => Response::Ok(None),
				}
			}
			Request::Remove{ key }=>{
				match store.remove(key){
					Ok(_) => Response::Ok(None),
					Err(e) => {
						error!("Remove failed: {}",e);
						Response::Err(e.to_string())
					}
				}
			}
			Request::Compact =>{
				println!("Compacting database...");
				match store.compact(){
					Ok(_) => {
						info!("Compaction triggered by user");
						Response::Ok(None)
					},
					Err(e) => {
						error!("Compaction failed: {}", e);
						Response::Err(e.to_string())
					}
				}
			}
		};
		
		if serde_json::to_writer(&stream, &response).is_err(){
			break;
		}
	}
	info!("Connection closed: {}", peer_addr);
}
