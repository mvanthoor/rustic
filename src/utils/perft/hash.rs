use crate::board::zobrist::ZobristKey;
use std::mem;

const BUCKET_SIZE: usize = 4;
const UNSET_HIGHEST_BYTE: u64 = 0x00_FF_FF_FF_FF_FF_FF_FF;

#[derive(Copy, Clone)]
pub struct PerftHashEntry {
    verification: u64,
    leaf_nodes: u64,
}

impl PerftHashEntry {
    pub fn new(v: u64, ln: u64) -> PerftHashEntry {
        PerftHashEntry {
            verification: v,
            leaf_nodes: ln,
        }
    }
}

#[derive(Clone)]
pub struct PerftHashBucket {
    bucket: [PerftHashEntry; BUCKET_SIZE],
}

impl PerftHashBucket {
    pub fn new() -> PerftHashBucket {
        PerftHashBucket {
            bucket: [PerftHashEntry::new(0, 0); BUCKET_SIZE],
        }
    }

    pub fn push(&mut self, entry: PerftHashEntry) {
        let mut index_with_lowest_depth = 0;
        for i in 1..BUCKET_SIZE {
            let bucket_depth = self.bucket[i].verification >> 56;
            let lowest_depth = self.bucket[index_with_lowest_depth].verification >> 56;
            if bucket_depth < lowest_depth {
                index_with_lowest_depth = i;
            }
        }
        self.bucket[index_with_lowest_depth] = entry;
    }

    pub fn find(&self, verification: u64) -> Option<u64> {
        for entry in self.bucket.iter() {
            let correct_verification = entry.verification == verification;
            if correct_verification {
                return Some(entry.leaf_nodes);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.bucket = [PerftHashEntry::new(0, 0); BUCKET_SIZE];
    }
}

pub struct PerftHashTable {
    hash_table: Vec<PerftHashBucket>,
    max_buckets: u64,
}

impl PerftHashTable {
    pub fn new(megabytes: u64) -> PerftHashTable {
        const BUCKET_MEMORY_USAGE: u64 = mem::size_of::<PerftHashBucket>() as u64;
        const BUCKETS_PER_MEGABYTE: u64 = (1024 * 1024 / BUCKET_MEMORY_USAGE);
        let buckets = BUCKETS_PER_MEGABYTE * megabytes;

        PerftHashTable {
            hash_table: vec![PerftHashBucket::new(); buckets as usize],
            max_buckets: buckets,
        }
    }

    pub fn clear(&mut self) {
        for bucket in self.hash_table.iter_mut() {
            bucket.clear();
        }
    }

    pub fn push(&mut self, depth: u8, zk: ZobristKey, leaf_nodes: u64) {
        let verification = (zk & UNSET_HIGHEST_BYTE) | ((depth as u64) << 56);
        let index = (zk % self.max_buckets) as usize;
        let entry = PerftHashEntry::new(verification, leaf_nodes);
        self.hash_table[index].push(entry);
    }

    pub fn leaf_nodes(&self, depth: u8, zk: ZobristKey) -> Option<u64> {
        let verification = (zk & UNSET_HIGHEST_BYTE) | ((depth as u64) << 56);
        let index = (zk % self.max_buckets) as usize;
        let bucket = &self.hash_table[index];

        bucket.find(verification)
    }
}
