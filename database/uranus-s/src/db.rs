use std::sync::{Arc, Mutex};

use uranus_kv::{StdHashKV, Storage};

#[derive(Debug, Clone)]
pub struct DBHandle {
    _storage: Arc<Mutex<dyn Storage + Send + Sync>>,
}

impl DBHandle {
    pub fn new() -> DBHandle {
        DBHandle {
            _storage: Arc::new(Mutex::new(StdHashKV::new())),
        }
    }
}

impl Default for DBHandle {
    fn default() -> Self {
        Self::new()
    }
}
