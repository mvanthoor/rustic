use crate::board::zobrist::ZobristKey;
use std::mem;

const BUCKET_SIZE: usize = 3;

#[derive(Copy, Clone)]
pub struct PerftHashEntry {
    depth: u8,
    zobrist_key: ZobristKey,
    leaf_nodes: u64,
}

impl PerftHashEntry {
    pub fn new(d: u8, zk: ZobristKey, ln: u64) -> PerftHashEntry {
        PerftHashEntry {
            depth: d,
            zobrist_key: zk,
            leaf_nodes: ln,
        }
    }
}

#[derive(Clone)]
pub struct PerftHashBucket {
    bucket: [PerftHashEntry; BUCKET_SIZE],
    in_use: u8,
}

impl PerftHashBucket {
    pub fn new() -> PerftHashBucket {
        PerftHashBucket {
            bucket: [PerftHashEntry::new(0, 0, 0); BUCKET_SIZE],
            in_use: 0,
        }
    }

    pub fn push(&mut self, entry: PerftHashEntry) {
        if (self.in_use as usize) < BUCKET_SIZE {
            self.bucket[self.in_use as usize] = entry;
            self.in_use += 1;
        } else {
            let mut index_with_lowest_depth = 0;
            for i in 1..BUCKET_SIZE {
                if self.bucket[i].depth < self.bucket[index_with_lowest_depth].depth {
                    index_with_lowest_depth = i;
                }
            }
            self.bucket[index_with_lowest_depth] = entry;
        }
    }

    pub fn find(&self, depth: u8, zk: ZobristKey) -> Option<u64> {
        for entry in self.bucket.iter() {
            let correct_key = entry.zobrist_key == zk;
            let correct_depth = entry.depth == depth;
            if correct_key && correct_depth {
                return Some(entry.leaf_nodes);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.bucket = [PerftHashEntry::new(0, 0, 0); BUCKET_SIZE];
    }
}

pub struct PerftHashTable {
    hash_table: Vec<PerftHashBucket>,
    max_entries: u64,
}

impl PerftHashTable {
    pub fn new(megabytes: u64) -> PerftHashTable {
        const BUCKET_MEMORY_USAGE: u64 = mem::size_of::<PerftHashBucket>() as u64;
        const BUCKETS_PER_MEGABYTE: u64 = (1024 * 1024 / BUCKET_MEMORY_USAGE);
        let buckets = BUCKETS_PER_MEGABYTE * megabytes;

        PerftHashTable {
            hash_table: vec![PerftHashBucket::new(); buckets as usize],
            max_entries: buckets,
        }
    }

    pub fn clear(&mut self) {
        for bucket in self.hash_table.iter_mut() {
            bucket.clear();
        }
    }

    pub fn push(&mut self, entry: PerftHashEntry) {
        let index = (entry.zobrist_key % self.max_entries) as usize;
        self.hash_table[index].push(entry);
    }

    pub fn leaf_nodes(&self, depth: u8, zk: ZobristKey) -> Option<u64> {
        let index = (zk % self.max_entries) as usize;
        let bucket = &self.hash_table[index];

        bucket.find(depth, zk)
    }
}
