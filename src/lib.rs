use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self,BufReader, BufWriter, Write};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use serde_json::Deserializer;
use std::io::Seek;
use log::{info, debug};

pub struct KvStore{
	map: HashMap<String,String>,
	writer: BufWriter<File>,
	path: PathBuf,
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
		
		let mut count = 0;
		for command in stream{
			match command?{
				Command::Set{key, value} => {
					map.insert(key, value);
				}
				Command::Remove{key} => {
					map.remove(&key);
				}
			}
			count+=1;
		}
		
		info!("Recovered {} commands from log file", count);
		debug!("Current keys in memory: {}", map.len());
		
		Ok(KvStore {
			map,
			writer: BufWriter::new(file),
			path,
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
	
	pub fn compact(&mut self) ->io::Result<()> {
		info!("Starting background compaction...");
		let mut compaction_path = self.path.clone();
		compaction_path.set_extension("rdb.tmp");
		
		let file = OpenOptions::new()
			.create(true)
			.write(true)
			.truncate(true)
			.open(&compaction_path)?;
		let mut new_writer = BufWriter::new(file);
		
		for (key, value) in &self.map {
			let cmd = Command::Set{
				key: key.clone(),
				value: value.clone()
			};
			serde_json::to_writer(&mut new_writer, &cmd)?;
		}
		new_writer.flush()?;
		
		fs::rename(&compaction_path, &self.path)?;
		
		let file = OpenOptions::new()
			.create(true)
			.write(true)
			.append(true)
			.open(&self.path)?;
		self.writer = BufWriter::new(file);
		
		info!("Compaction completed. Old log file removed");
		Ok(())
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request{
	Get {key : String},
	Set {key: String, value: String},
	Remove {key: String},
	Compact,
}

#[derive(Debug, Serialize,Deserialize)]
pub enum Response {
	Ok(Option<String>), 
	Err(String),
}

