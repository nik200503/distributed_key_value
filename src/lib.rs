use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write, Seek, SeekFrom};
use std::path::PathBuf;
use std::collections::BTreeMap;

pub struct KvStore {
    map: BTreeMap<String, String>,
    writer: BufWriter<File>,
    path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request{
    Get { key: String},
    Set { key: String, value:String},
    Remove { key: String},
    Compact,
    Scan { start: String, end: String},
    ReplicateSet { key: String, value: String},
    ReplicateRm {key: String},
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Ok(Option<String>),
    Err(String),
    ScanResult(Vec<(String, String)>),
}

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl KvStore {
    pub fn open(path: PathBuf) -> std::io::Result<KvStore> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)?;

        let mut map = BTreeMap::new();

        let mut reader = BufReader::new(File::open(&path)?);

        loop {
            let pos = reader.stream_position()?; 
            match bincode::deserialize_from::<_, Command>(&mut reader) {
                Ok(command) => {
                    match command {
                        Command::Set { key, value } => { map.insert(key, value); }
                        Command::Remove { key } => { map.remove(&key); }
                    }
                }
                Err(_) => break, 
            }
        }

        Ok(KvStore {
            map,
            writer: BufWriter::new(file),
            path,
        })
    }

    pub fn set(&mut self, key: String, value: String) -> std::io::Result<()> {
        let cmd = Command::Set {
            key: key.clone(),
            value: value.clone(),
        };

        bincode::serialize_into(&mut self.writer, &cmd).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.writer.flush()?;

        self.map.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).cloned()
    }

    pub fn remove(&mut self, key: String) -> std::io::Result<()> {
        if self.map.contains_key(&key) {
            let cmd = Command::Remove { key: key.clone() };

            bincode::serialize_into(&mut self.writer, &cmd).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            self.writer.flush()?;

            self.map.remove(&key);
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "key not found"))
        }
    }

    pub fn compact(&mut self) -> io::Result<()> {
        let mut compaction_path = self.path.clone();
        compaction_path.set_extension("rdb.tmp");

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&compaction_path)?;
        let mut new_writer = BufWriter::new(file);

        for (key, value) in &self.map {
            let cmd = Command::Set {
                key: key.clone(),
                value: value.clone(),
            };
            bincode::serialize_into(&mut self.writer, &cmd).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }
        new_writer.flush()?;

        fs::rename(&compaction_path, &self.path)?;

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        self.writer = BufWriter::new(file);

        Ok(())
    }
    pub fn scan(&self, start: String, end: String) -> Vec<(String, String)>{
        self.map
            .range(start..end)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
