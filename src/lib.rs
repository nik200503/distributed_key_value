use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write, Seek, SeekFrom};
use std::path::PathBuf;
use std::collections::BTreeMap;
use crc32fast::Hasher;
use std::io::Read;
use snap::raw::{Decoder, Encoder};

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
            let checksum: u32 = match bincode::deserialize_from(&mut reader){
                Ok(c) => c,
                Err(_) => break,
            };
            
            let len:u64 = match bincode::deserialize_from(&mut reader){
                Ok(l) => l,
                Err(_) => break,
            };
            
            let mut compressed_payload = vec![0u8; len as usize];
            if let Err(_) = reader.read_exact(&mut compressed_payload){ 
                break;
            }
            
            let mut hasher = Hasher::new();
            hasher.update(&compressed_payload);
            if hasher.finalize() != checksum {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Data Corruption detected: CRC mismatch"));
            }
            
            let raw_payload = Decoder::new()
                .decompress_vec(&compressed_payload)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            
            let command: Command = bincode::deserialize(&raw_payload)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            match command {
                Command::Set { key, value } => {
                    map.insert(key, value);
                }
                Command::Remove { key } => {
                    map.remove(&key);
                }
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
        
        self.save_record(&cmd)?;

        self.map.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).cloned()
    }

    pub fn remove(&mut self, key: String) -> std::io::Result<()> {
        if self.map.contains_key(&key) {
            let cmd = Command::Remove { key: key.clone() };
            self.save_record(&cmd)?;
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
    
    fn save_record(&mut self, cmd: &Command) -> io::Result<()>{
        let raw_payload = bincode::serialize(cmd)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let compressed_payload = Encoder::new()
            .compress_vec(&raw_payload)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        let mut hasher = Hasher::new();
        hasher.update(&compressed_payload);
        let checksum = hasher.finalize();
        
        let len = compressed_payload.len() as u64;
        
        bincode::serialize_into(&mut self.writer, &checksum)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        bincode::serialize_into(&mut self.writer, &len)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            
        self.writer.write_all(&compressed_payload)?;
        self.writer.flush()?;
        
        Ok(())
    }
}
