/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

use crate::board::defs::ZobristKey;

const MEGABYTE: usize = 1024 * 1024;
const ENTRIES_PER_BUCKET: usize = 4;
const HIGH_FOUR_BYTES: usize = 0xFF_FF_FF_FF_00_00_00_00;
const LOW_FOUR_BYTES: usize = 0x00_00_00_00_FF_FF_FF_FF;
const SHIFT_TO_LOWER: usize = 32;

// ===== Data ==================================================================================//

pub trait IHashData {
    fn new() -> Self;
    fn depth(&self) -> u8;
}
#[derive(Copy, Clone)]
pub struct PerftData {
    pub leaf_nodes: u64,
    pub depth: u8,
}

impl IHashData for PerftData {
    fn new() -> Self {
        Self {
            leaf_nodes: 0,
            depth: 0,
        }
    }

    fn depth(&self) -> u8 {
        self.depth
    }
}

#[derive(Copy, Clone)]
pub struct SearchData {
    pub stuff1: u32,
    pub depth: u8,
}

impl IHashData for SearchData {
    fn new() -> Self {
        Self {
            stuff1: 0,
            depth: 0,
        }
    }

    fn depth(&self) -> u8 {
        self.depth
    }
}

// ===== Entry ==================================================================================//

#[derive(Copy, Clone)]
struct Entry<D> {
    verification: u32,
    data: D,
}

impl<D: IHashData> Entry<D> {
    pub fn new() -> Self {
        Self {
            verification: 0,
            data: D::new(),
        }
    }
}

// ===== Bucket ================================================================================ //

#[derive(Clone)]
struct Bucket<D> {
    bucket: [Entry<D>; ENTRIES_PER_BUCKET],
}

impl<D: IHashData + Copy> Bucket<D> {
    pub fn new() -> Self {
        Self {
            bucket: [Entry::new(); ENTRIES_PER_BUCKET],
        }
    }

    pub fn store(&mut self, verification: u32, data: D) {
        let mut idx_lowest_depth = 0;
        for entry in 1..ENTRIES_PER_BUCKET {
            if self.bucket[entry].data.depth() < data.depth() {
                idx_lowest_depth = entry
            }
        }

        self.bucket[idx_lowest_depth] = Entry { verification, data }
    }
}

// ===== Hash table ============================================================================ //

pub struct HashTable<D> {
    hash_table: Vec<Bucket<D>>,
    megabytes: usize,
    used_entries: usize,
    total_buckets: usize,
    total_entries: usize,
}

// Public functions
impl<D: IHashData + Copy + Clone> HashTable<D> {
    pub fn new(megabytes: usize) -> Self {
        let entry_size = std::mem::size_of::<Entry<D>>();
        let bucket_size = entry_size * ENTRIES_PER_BUCKET;
        let total_buckets = MEGABYTE / bucket_size * megabytes;
        let total_entries = total_buckets * ENTRIES_PER_BUCKET;

        Self {
            hash_table: vec![Bucket::<D>::new(); total_buckets],
            megabytes,
            used_entries: 0,
            total_buckets,
            total_entries,
        }
    }

    pub fn insert(&mut self, zobrist_key: ZobristKey, data: D) {
        if self.megabytes > 0 {
            let index = self.calculate_index(zobrist_key);
            let verification = self.calculate_verification(zobrist_key);
            self.hash_table[index].store(verification, data);
        }
    }

    // Sends hash usage in permille.
    pub fn hash_full(&self) -> u16 {
        ((self.used_entries * 1000) / self.total_entries) as u16
    }
}

// Private functions
impl<D: IHashData + Copy + Clone> HashTable<D> {
    fn calculate_index(&self, zobrist_key: ZobristKey) -> usize {
        ((zobrist_key as usize & HIGH_FOUR_BYTES) >> SHIFT_TO_LOWER) % self.total_buckets
    }

    fn calculate_verification(&self, zobrist_key: ZobristKey) -> u32 {
        ((zobrist_key as usize) & LOW_FOUR_BYTES) as u32
    }
}
