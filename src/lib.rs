use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KvStore{
	map: HashMap<String,String>,
	#[serde(skip)]
	path: PathBuf,
}

impl KvStore{
	pub fn open(path: PathBuf) -> std::io::Result<KvStore>{
		let file_result = fs::read_to_string(&path);
		
		match file_result{
			Ok(contents) => {
				let mut store: KvStore = serde_json::from_str(&contents)
							.unwrap_or_else(|_| KvStore {
							map: HashMap::new(),
							path: path.clone(),
				});
			store.path = path;
			Ok(store)
			},
			Err(_) => {
				Ok(KvStore {
					map: HashMap::new(),
					path,
				})
			}
		}
	}
	
	pub fn set(&mut self, key: String, value: String)->std::io::Result<()>{
		self.map.insert(key,value);
		self.save()?;
		Ok(())
	}
	
	pub fn get(&self, key: String)-> Option<String> {
		self.map.get(&key).cloned()
	}
	
	pub fn remove(&mut self,key: String)-> std::io::Result<()>{
		self.map.remove(&key);
		self.save()?;
		Ok(())
	}
	
	fn save(&self) -> std::io::Result<()>{
		let contents = serde_json::to_string_pretty(&self)?;
		fs::write(&self.path, contents)?;
		Ok(())
	}
	
}


