//! An implementation of LevelDB's memtable in Rust
//!
//! The original file: /db/skiplist.h
//!

use std::sync::atomic::AtomicPtr;

use bytes::Bytes;

struct _Node {
    key: Bytes,
    next: AtomicPtr<_Node>,
}

struct _SkipList {}
