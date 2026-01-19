use clap::{Parser, ValueEnum};
use log::{LevelFilter, error, info};
use rust_kv::{KvStore, Request, Response};
use serde::Deserialize;
use serde_json::de::Deserializer;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::process::exit;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ServerRole {
    Leader,
    Follower,
}

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct ServerCli {
    #[arg(long, default_value = "127.0.0.1")]
    addr: String,

    #[arg(long, default_value = "4000")]
    port: u16,

    #[arg(long, value_enum, default_value_t= ServerRole::Leader)]
    role: ServerRole,

    #[arg(long)]
    leader_addr: Option<String>,
}

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let args = ServerCli::parse();

    match args.role {
        ServerRole::Follower => {
            if args.leader_addr.is_none() {
                error!("Error: A follower must know the leader's address! Use --leader-addr");
                exit(1);
            }
            info!(
                "Starting as Follower. Leader is at {:?}",
                args.leader_addr.unwrap()
            );
        }
        ServerRole::Leader => {
            info!("Starting as LEADER. I am the source of truth.");
        }
    }

    let bind_addr = format!("{}:{}", args.addr, args.port);
    info!("Listening on {}", bind_addr);

    let db_path = format!("kv_{}.db", args.port);
    info!("using database file: {}", db_path);

    let store = KvStore::open(PathBuf::from("kv.db")).expect("failed to open db");
    let shared_store = Arc::new(Mutex::new(store));

    let listener = TcpListener::bind(&bind_addr).expect("Failed to bind");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store_handle = shared_store.clone();
                let current_role = args.role;
                thread::spawn(move || {
                    handle_connection(stream, store_handle, current_role);
                });
            }
            Err(e) => error!("connection Failed : {}", e),
        }
    }
}

fn handle_connection(stream: TcpStream, db: Arc<Mutex<KvStore>>, role: ServerRole) {
    let peer_addr = stream.peer_addr().unwrap();
    info!("New connection from {}", peer_addr);

    let mut stream_de = Deserializer::from_reader(&stream);

    while let Ok(request) = Request::deserialize(&mut stream_de) {
        let mut store = db.lock().unwrap();

        let response = match request {
            Request::Set { .. } | Request::Remove { .. } | Request::Compact => {
                if role == ServerRole::Follower {
                    Response::Err(
                        "Write command rejected: I am a Follower. Connect to Leader.".to_string(),
                    )
                } else {
                    match request {
                        Request::Set { key, value } => match store.set(key, value) {
                            Ok(_) => Response::Ok(None),
                            Err(e) => Response::Err(e.to_string()),
                        },
                        Request::Remove { key } => match store.remove(key) {
                            Ok(_) => Response::Ok(None),
                            Err(e) => Response::Err(e.to_string()),
                        },
                        Request::Compact => match store.compact() {
                            Ok(_) => Response::Ok(None),
                            Err(e) => Response::Err(e.to_string()),
                        },
                        _ => Response::Err("Internal Error".to_string()),
                    }
                }
            }
            Request::Get { key } => match store.get(key) {
                Some(val) => Response::Ok(Some(val)),
                None => Response::Ok(None),
            },
        };

        if serde_json::to_writer(&stream, &response).is_err() {
            break;
        }
    }
}
