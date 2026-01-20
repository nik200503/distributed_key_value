use clap::{Parser, ValueEnum};
use log::{LevelFilter, error, info};
use rust_kv::{KvStore, Request, Response};
use serde::Deserialize;
use serde_json::de::Deserializer;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;

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

    #[arg(long)]
    follower_addr: Option<String>,
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
                let follower_addr = args.follower_addr.clone();
                thread::spawn(move || {
                    handle_connection(stream, store_handle, current_role, follower_addr);
                });
            }
            Err(e) => error!("connection Failed : {}", e),
        }
    }
}

fn handle_connection(
    stream: TcpStream,
    db: Arc<Mutex<KvStore>>,
    role: ServerRole,
    follower_addr: Option<String>,
) {
    let peer_addr = stream.peer_addr().unwrap();
    let mut stream_de = Deserializer::from_reader(&stream);

    while let Ok(request) = Request::deserialize(&mut stream_de) {
        let mut store = db.lock().unwrap();

        let response = match request {
            Request::Set { ref key, ref value } => {
                if role == ServerRole::Follower {
                    Response::Err("Write command rejected: connect to leader.".to_string())
                } else {
                    match store.set(key.clone(), value.clone()) {
                        Ok(_) => {
                            if let Some(addr) = &follower_addr {
                                replicate_to_follower(
                                    addr,
                                    Request::Set {
                                        key: key.clone(),
                                        value: value.clone(),
                                    },
                                );
                            }
                            Response::Ok(None)
                        }
                        Err(e) => Response::Err(e.to_string()),
                    }
                }
            }
            Request::Remove { ref key } => {
                if role == ServerRole::Follower {
                    Response::Err("Write command rejected.".to_string())
                } else {
                    match store.remove(key.clone()) {
                        Ok(_) => {
                            if let Some(addr) = &follower_addr {
                                replicate_to_follower(addr, Request::Remove { key: key.clone() });
                            }
                            Response::Ok(None)
                        }
                        Err(e) => Response::Err(e.to_string()),
                    }
                }
            }
            Request::ReplicateSet { key, value } => match store.set(key, value) {
                Ok(_) => Response::Ok(None),
                Err(e) => Response::Err(e.to_string()),
            },
            Request::ReplicateRm { key } => match store.remove(key) {
                Ok(_) => Response::Ok(None),
                Err(e) => Response::Err(e.to_string()),
            },
            Request::Get { key } => match store.get(key) {
                Some(val) => Response::Ok(Some(val)),
                None => Response::Ok(None),
            },
            _ => Response::Ok(None),
        };

        if serde_json::to_writer(&stream, &response).is_err() {
            break;
        }
    }
}

fn replicate_to_follower(follower_addr: &str, req: Request) {
    let repl_req = match req {
        Request::Set { key, value } => Request::ReplicateSet { key, value },
        Request::Remove { key } => Request::ReplicateRm { key },
        _ => return,
    };

    match TcpStream::connect(follower_addr) {
        Ok(stream) => {
            info!("Replicating to follower at {}", follower_addr);

            if let Err(e) = serde_json::to_writer(&stream, &repl_req) {
                error!("Failed to send replication data: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to connect to Follower at {}: {}", follower_addr, e);
        }
    }
}
