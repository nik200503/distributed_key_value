use r2d2::ManageConnection;
use r2d2::Pool;
use rouille::{Response, router};
use rust_kv::{Request, Response as KvResponse};
use serde::Deserialize;
use serde_json::de::Deserializer;
use std::io::prelude::*;
use std::net::TcpStream;

struct KvConnectionManager {
    addr: String,
}

impl ManageConnection for KvConnectionManager {
    type Connection = TcpStream;
    type Error = std::io::Error;

    fn connect(&self) -> Result<TcpStream, std::io::Error> {
        TcpStream::connect(&self.addr)
    }

    fn is_valid(&self, _conn: &mut TcpStream) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut TcpStream) -> bool {
        false
    }
}

fn main() {
    println!("HTTP Gateway listening on localhost:8080");

    let manager = KvConnectionManager {
        addr: "127.0.0.1:4000".to_string(),
    };

    let pool = Pool::builder()
        .max_size(15)
        .build(manager)
        .expect("Failed to create pool");

    rouille::start_server("0.0.0.0:8080", move |request| {
        let mut conn = pool.get().expect("Failed to get connection from pool");

        router!(request,
            (GET) (/{key: String})=> {
                let kv_req = Request::Get {key};
                match send_to_db(&mut conn , kv_req){
                    Ok(Some(value))=> Response::text(value),
                    Ok(None) => Response::text("key not found").with_status_code(404),
                    Err(e) => Response::text(format!("DB Error {}", e)).with_status_code(500),
                }
            },

            (POST) (/{key: String}) => {
                let mut data = request.data().expect("Body is corrupted");
                let mut value = String::new();
                data.read_to_string(&mut value).unwrap();

                let kv_req = Request::Set {key, value};
                match send_to_db(&mut conn,kv_req){
                    Ok(_) => Response::text("Success"),
                    Err(e) => Response::text(format!("DB Error: {}", e)).with_status_code(500),
                }
            },

            _ => Response::empty_404()
        )
    });
}

fn send_to_db(stream: &mut TcpStream, req: Request) -> Result<Option<String>, String> {
    serde_json::to_writer(stream.try_clone().unwrap(), &req).map_err(|e| e.to_string())?;

    let mut stream_de = Deserializer::from_reader(stream);
    let resp = KvResponse::deserialize(&mut stream_de).map_err(|e| e.to_string())?;

    match resp {
        KvResponse::Ok(val) => Ok(val),
        KvResponse::Err(e) => Err(e),
    }
}
