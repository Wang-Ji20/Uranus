use std::sync::{Arc, Mutex};

use anyhow::Result;
use bytes::Bytes;
use uranus_kv::{StdHashKV, Storage};

#[derive(Debug, Clone)]
pub struct DBHandle {
    storage: Arc<Mutex<dyn Storage + Send + Sync>>,
}

impl DBHandle {
    pub fn new() -> DBHandle {
        DBHandle {
            storage: Arc::new(Mutex::new(StdHashKV::new())),
        }
    }

    pub fn get(&self, key: impl Into<Bytes>) -> Result<Option<Bytes>> {
        let db = self.storage.lock().unwrap();
        db.get(key.into())
    }

    pub fn put(&self, key: impl Into<Bytes>, value: impl Into<Bytes>) -> Result<()> {
        let mut db = self.storage.lock().unwrap();
        db.put(key.into(), value.into())
    }
}

impl Default for DBHandle {
    fn default() -> Self {
        Self::new()
    }
}
