use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self,BufReader, BufWriter, Write};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use serde_json::Deserializer;

pub struct KvStore{
	map: HashMap<String,String>,
	writer: BufWriter<File>,
}

#[derive(Serialize, Deserialize, Debug)]
enum Command{
	Set {key: String, value: String},
	Remove{key: String},
}

impl KvStore{
	pub fn open(path: PathBuf) -> std::io::Result<KvStore>{
		let file = OpenOptions::new()
			.create(true)
			.write(true)
			.append(true)
			.read(true)
			.open(&path)?;
		
		let mut map = HashMap::new();
		
		let reader = BufReader::new(File::open(&path)?);
		
		let stream = Deserializer::from_reader(reader).into_iter::<Command>();
		
		for command in stream{
			match command?{
				Command::Set{key, value} => {
					map.insert(key, value);
				}
				Command::Remove{key} => {
					map.remove(&key);
				}
			}
		}
		
		Ok(KvStore {
			map,
			writer: BufWriter::new(file),
		})
	}
	
	pub fn set(&mut self, key: String, value: String)->std::io::Result<()>{
		let cmd = Command::Set{
			key:key.clone(),
			value: value.clone()
		};
		
		serde_json::to_writer(&mut self.writer, &cmd)?;
		self.writer.flush()?;
		
		
		self.map.insert(key,value);
		Ok(())
	}
	
	pub fn get(&self, key: String)-> Option<String> {
		self.map.get(&key).cloned()
	}
	
	pub fn remove(&mut self,key: String)-> std::io::Result<()>{
		if self.map.contains_key(&key){
			let cmd = Command::Remove{ key:key.clone()};
			
			serde_json::to_writer(&mut self.writer, &cmd)?;
			self.writer.flush()?;
			
			self.map.remove(&key);
			Ok(())
		}else{
			Err(io::Error::new(io::ErrorKind::NotFound, "key not found"))
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request{
	Get {key : String},
	Set {key: String, value: String},
	Remove {key: String},
}

#[derive(Debug, Serialize,Deserialize)]
pub enum Response {
	Ok(Option<String>), 
	Err(String),
}

