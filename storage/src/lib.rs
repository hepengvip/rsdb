extern crate rocksdb;
extern crate tempfile;

use rocksdb::{Options, DB};


pub struct Storage {
    pub db: DB,
}

impl Storage {
    pub fn new(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path).unwrap();
        Self { db }
    }

    pub fn new_with_temp_dir(prefix: &str) -> Self {
        let dir = tempfile::Builder::new()
           .prefix(prefix)
           .tempdir()
           .unwrap();
        let db = DB::open_default(dir.path()).unwrap();
        Self { db }
    }

    pub fn set(&self, key: &[u8], value: &[u8]) {
        self.db.put(key, value).unwrap();
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>>{
        self.db.get(key).unwrap()
    }

    pub fn delete(&self, key: &[u8]) {
        self.db.delete(key).unwrap();
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
