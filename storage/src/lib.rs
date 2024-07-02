use std::collections::HashMap;
use std::mem::drop;
use std::sync::Arc;

extern crate rocksdb;
extern crate tempfile;

pub use rocksdb::{Direction, IteratorMode};
use rocksdb::{Options, DB};

pub struct MultiDB {
    storage: HashMap<String, Arc<Storage>>,
    root_path: String,
}

impl MultiDB {
    pub fn new(root_path: &str) -> Self {
        Self {
            storage: HashMap::new(),
            root_path: root_path.to_string(),
        }
    }

    pub fn get_db(&self, name: &str) -> Option<Arc<Storage>> {
        let s = self.storage.get(name);
        if let Some(s) = s {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn attach(&mut self, name: &str) {
        let s_opt = self.get_db(name);
        if let Some(_s) = s_opt {
            return;
        }
        let db_path = format!("{}/{}", self.root_path, name);
        // let db = DB::open_default(&db_path).unwrap();
        let storage = Storage::new(&db_path);
        self.storage.insert(name.to_string(), Arc::new(storage));
    }

    pub fn detach(&mut self, name: &str) {
        let s_opt = self.storage.remove(name);
        if let Some(s) = s_opt {
            drop(s);
        }
    }

    pub fn list_db(&self) -> Vec<&[u8]> {
        self.storage.iter().map(|(k, _v)| k.as_bytes()).collect()
    }
}

pub struct Storage {
    pub db: DB,
    pub path: Option<String>,
    pub temp: bool,
}

impl Storage {
    pub fn new(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path).unwrap();
        Self {
            db,
            path: Some(path.to_string()),
            temp: false,
        }
    }

    pub fn new_with_temp_dir(prefix: &str) -> Self {
        let dir = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
        let db = DB::open_default(dir.path()).unwrap();
        Self {
            db,
            path: None,
            temp: true,
        }
    }

    pub fn set(&self, key: &[u8], value: &[u8]) {
        self.db.put(key, value).unwrap();
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.db.get(key).unwrap()
    }

    pub fn delete(&self, key: &[u8]) {
        self.db.delete(key).unwrap();
    }

    pub fn this_db(&self) -> &DB {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage() {
        let storage = Storage::new_with_temp_dir("test_storage");
        storage.set(b"key1", b"value1");
        assert_eq!(storage.get(b"key1").unwrap(), b"value1");
        storage.delete(b"key1");
        assert_eq!(storage.get(b"key1"), None);
    }
}
