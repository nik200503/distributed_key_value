use rouille::{router, Response};
use std::io::prelude::*;
use std::net::TcpStream;
use rust_kv::{Request, Response as KvResponse};
use serde_json::de::Deserializer;
use serde::Deserialize;

fn main(){
    println!("HTTP Gateway listening on localhost:8080");
    
    rouille::start_server("0.0.0.0:8080", move |request| {
        router!(request,
            (GET) (/{key: String})=> {
                let kv_req = Request::Get {key};
                match send_to_db(kv_req){
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
                match send_to_db(kv_req){
                    Ok(_) => Response::text("Success"),
                    Err(e) => Response::text(format!("DB Error: {}", e)).with_status_code(500),
                }
            },
            
            _ => Response::empty_404()
        )
    });
}


fn send_to_db(req: Request) -> Result<Option<String>, String>{
    let stream = TcpStream::connect("127.0.0.1:4000").map_err(|e| e.to_string())?;
    
    serde_json::to_writer(&stream, &req).map_err(|e| e.to_string())?;
    
    let mut stream_de = Deserializer::from_reader(&stream);
    let resp = KvResponse::deserialize(&mut stream_de).map_err(|e| e.to_string())?;
    
    match resp{
        KvResponse::Ok(val) => Ok(val),
        KvResponse::Err(e) => Err(e),
    }
}
