use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::io::Error as IOError;
use std::mem::drop;
use std::sync::Arc;

extern crate rocksdb;
extern crate tempfile;

pub use rocksdb::{Direction, IteratorMode};
use rocksdb::{Error as DBError, Options, DB};

pub struct MultiDB {
    storage: HashMap<String, Arc<Storage>>,
    root_path: String,
}

#[derive(Debug)]
pub enum StorageError {
    DbErr(DBError),
    IoErr(IOError),
}

impl Error for StorageError {}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StorageError::DbErr(e) => write!(f, "DBError: {}", e),
            StorageError::IoErr(e) => write!(f, "IOError: {}", e),
        }
    }
}

impl From<DBError> for StorageError {
    fn from(e: DBError) -> Self {
        StorageError::DbErr(e)
    }
}

impl From<IOError> for StorageError {
    fn from(e: IOError) -> Self {
        StorageError::IoErr(e)
    }
}

pub type StorageResult<T> = Result<T, StorageError>;

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

    pub fn attach(&mut self, name: &str) -> StorageResult<()> {
        let s_opt = self.get_db(name);
        if let Some(_s) = s_opt {
            return Ok(());
        }
        let db_path = format!("{}/{}", self.root_path, name);
        let storage = Storage::new(&db_path)?;
        self.storage.insert(name.to_string(), Arc::new(storage));
        Ok(())
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
    pub fn new(path: &str) -> StorageResult<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Self {
            db,
            path: Some(path.to_string()),
            temp: false,
        })
    }

    pub fn new_with_temp_dir(prefix: &str) -> StorageResult<Self> {
        let dir = tempfile::Builder::new().prefix(prefix).tempdir()?;
        let db = DB::open_default(dir.path())?;
        Ok(Self {
            db,
            path: None,
            temp: true,
        })
    }

    pub fn set(&self, key: &[u8], value: &[u8]) -> StorageResult<()> {
        self.db.put(key, value)?;
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        Ok(self.db.get(key)?)
    }

    pub fn delete(&self, key: &[u8]) -> StorageResult<()> {
        Ok(self.db.delete(key)?)
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
        let storage = Storage::new_with_temp_dir("test_storage").unwrap();
        storage.set(b"key1", b"value1").unwrap();
        assert_eq!(storage.get(b"key1").unwrap().unwrap(), b"value1");
        storage.delete(b"key1").unwrap();
        assert_eq!(storage.get(b"key1").unwrap(), None);
    }
}
