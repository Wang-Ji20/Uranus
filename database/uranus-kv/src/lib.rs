use std::{collections::HashMap, fmt::Debug};

use anyhow::Result;
use bytes::Bytes;
use thiserror::Error;

pub trait Storage {
    fn put(&mut self, key: Bytes, value: Bytes) -> Result<()>;
    fn delete(&mut self, key: Bytes) -> Result<()>;
    fn get(&self, key: Bytes) -> Result<Option<Bytes>>;
}

impl Debug for dyn Storage + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Something")
    }
}

pub struct StdHashKV {
    hashmap: HashMap<Bytes, Bytes>,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("put failed")]
    PutFailed,
    #[error("delete failed")]
    DeleteFailed,
    #[error("get failed")]
    GetFailed,
}

impl Storage for StdHashKV {
    /// put here is almost always succeed, but for other storage systems that may not be the case..
    fn put(&mut self, key: Bytes, value: Bytes) -> Result<()> {
        self.hashmap.insert(key, value);
        Ok(())
    }

    fn delete(&mut self, key: Bytes) -> Result<()> {
        self.hashmap
            .remove(&key)
            .ok_or(StorageError::DeleteFailed)?;
        Ok(())
    }

    fn get(&self, key: Bytes) -> Result<Option<Bytes>> {
        let result = self.hashmap.get(&key).map(|x| x.to_owned());
        Ok(result)
    }
}

impl Default for StdHashKV {
    fn default() -> Self {
        Self::new()
    }
}

impl StdHashKV {
    pub fn new() -> StdHashKV {
        StdHashKV {
            hashmap: HashMap::new(),
        }
    }
}

pub struct KV {}

impl Storage for KV {
    fn put(&mut self, _: Bytes, _: Bytes) -> Result<()> {
        todo!()
    }

    fn delete(&mut self, _: Bytes) -> Result<()> {
        todo!()
    }

    fn get(&self, _: Bytes) -> Result<Option<Bytes>> {
        todo!()
    }
}

pub mod arena;
pub mod memtable;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
