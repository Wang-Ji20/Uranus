use anyhow::Result;
use bytes::Bytes;

pub trait Storage {
    fn put(key: &Bytes, value: &Bytes) -> Result<()>;
    fn delete(key: &Bytes) -> Result<()>;
    fn get(key: &Bytes) -> Result<Bytes>;
}

pub struct KV {}

impl Storage for KV {
    fn put(_: &Bytes, _: &Bytes) -> Result<()> {
        todo!()
    }

    fn delete(_: &Bytes) -> Result<()> {
        todo!()
    }

    fn get(_: &Bytes) -> Result<Bytes> {
        todo!()
    }
}

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
