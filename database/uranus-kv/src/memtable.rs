//! A re-implementation of LevelDB's memtable in Rust
//!
//! The original file: /db/skiplist.h
//!

use bytes::Bytes;

type _NodeDescriptor = usize;

struct _Node {
    key: Bytes,
    next: [_NodeDescriptor],
}

struct _SkipList {}
