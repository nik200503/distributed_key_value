mod lib;
use lib::KvStore;

fn main() {
    let mut store = KvStore::new();
    
    store.set("key1".to_string(), "value1".to_string());
    
    match store.get("key1".to_string()){
    	Some(value)=> println!("Found it {value}"),
    	None => println!("key not found"),
    }
    
    store.remove("key1".to_string());
}
