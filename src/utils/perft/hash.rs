use crate::board::zobrist::ZobristKey;
use std::mem;

#[derive(Clone)]
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

pub struct PerftHashTable {
    hash_table: Vec<PerftHashEntry>,
    max_entries: u64,
    count: u64,
}

impl PerftHashTable {
    pub fn new(megabytes: u64) -> PerftHashTable {
        const ENTRY_SIZE: u64 = mem::size_of::<PerftHashEntry>() as u64;
        const ENTRIES_PER_MEGABYTE: u64 = (1024 * 1024 / ENTRY_SIZE);
        let entries = ENTRIES_PER_MEGABYTE * megabytes;
        PerftHashTable {
            hash_table: vec![PerftHashEntry::new(0, 0, 0); entries as usize],
            max_entries: entries,
            count: 0,
        }
    }

    pub fn clear(&mut self) {
        self.hash_table = vec![PerftHashEntry::new(0, 0, 0); self.max_entries as usize];
        self.count = 0;
    }

    pub fn push(&mut self, entry: PerftHashEntry) {
        let index = (entry.zobrist_key % self.max_entries) as usize;
        self.hash_table[index] = entry;
    }

    pub fn leaf_nodes(&self, depth: u8, zk: ZobristKey) -> Option<u64> {
        let index = (zk % self.max_entries) as usize;
        let correct_key = self.hash_table[index].zobrist_key == zk;
        let correct_depth = self.hash_table[index].depth == depth;
        if correct_key && correct_depth {
            Some(self.hash_table[index].leaf_nodes)
        } else {
            None
        }
    }
}
