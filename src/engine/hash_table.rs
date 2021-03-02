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

/* ===== Data ========================================================= */

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

impl PerftData {
    pub fn get_value(&self, depth: u8) -> Option<u64> {
        if self.depth == depth {
            Some(self.leaf_nodes)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub enum HashFlags {
    NONE,
    EXACT,
    ALPHA,
    BETA,
}

#[derive(Copy, Clone)]
pub struct SearchData {
    pub depth: u8,
    pub flags: HashFlags,
    pub eval: i16,
}

impl IHashData for SearchData {
    fn new() -> Self {
        Self {
            depth: 0,
            flags: HashFlags::NONE,
            eval: 0,
        }
    }

    fn depth(&self) -> u8 {
        self.depth
    }
}

/* ===== Entry ======================================================== */

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

/* ===== Bucket ======================================================= */

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

    // Store a position in the bucket. Replace the position with the stored
    // lowest depth, as positions with higher depth are more valuable.
    pub fn store(&mut self, verification: u32, data: D, used_entries: &mut usize) {
        let mut idx_lowest_depth = 0;

        // Find the index of the entry with the lowest depth.
        for entry in 1..ENTRIES_PER_BUCKET {
            if self.bucket[entry].data.depth() < data.depth() {
                idx_lowest_depth = entry
            }
        }

        // If the verifiaction was 0, this entry in the bucket was never
        // used before. Count the use of this entry.
        if self.bucket[idx_lowest_depth].verification == 0 {
            *used_entries += 1;
        }

        // Store.
        self.bucket[idx_lowest_depth] = Entry { verification, data }
    }

    // Find a position in the bucket, where both the stored verification and
    // depth match the requested verification and depth.
    pub fn find(&self, verification: u32) -> Option<&D> {
        for e in self.bucket.iter() {
            if e.verification == verification {
                return Some(&e.data);
            }
        }
        None
    }
}

/* ===== Hash table =================================================== */

pub struct HashTable<D> {
    hash_table: Vec<Bucket<D>>,
    megabytes: usize,
    used_entries: usize,
    total_buckets: usize,
    total_entries: usize,
}

// Public functions
impl<D: IHashData + Copy + Clone> HashTable<D> {
    // Create a new hash table of the requested size, able to hold the data
    // of type D, where D has to implement IHashData, and must be clonable
    // and copyable.
    pub fn new(megabytes: usize) -> Self {
        let (total_buckets, total_entries) = Self::calculate_init_values(megabytes);

        Self {
            hash_table: vec![Bucket::<D>::new(); total_buckets],
            megabytes,
            used_entries: 0,
            total_buckets,
            total_entries,
        }
    }

    // Resizes the hash table by replacing the current hash table with a
    // new one. (We don't use Vec's resize function, because it clones
    // elements. This can be problematic if hash table sizes push the
    // computer's memory limits.)
    pub fn resize(&mut self, megabytes: usize) {
        let (total_buckets, total_entries) = HashTable::<D>::calculate_init_values(megabytes);

        self.hash_table = vec![Bucket::<D>::new(); total_buckets];
        self.megabytes = megabytes;
        self.used_entries = 0;
        self.total_buckets = total_buckets;
        self.total_entries = total_entries;
    }

    // Insert a position at the calculated index, by storing it in the
    // index's bucket.
    pub fn insert(&mut self, zobrist_key: ZobristKey, data: D) {
        if self.megabytes > 0 {
            let index = self.calculate_index(zobrist_key);
            let verification = self.calculate_verification(zobrist_key);
            self.hash_table[index].store(verification, data, &mut self.used_entries);
        }
    }

    // Probe the hash table by both verification and depth. Both have to
    // match for the position to be the correct one we're looking for.
    pub fn probe(&self, zobrist_key: ZobristKey) -> Option<&D> {
        if self.megabytes > 0 {
            let index = self.calculate_index(zobrist_key);
            let verification = self.calculate_verification(zobrist_key);

            self.hash_table[index].find(verification)
        } else {
            None
        }
    }

    // Clear hash table by replacing it with a new one.
    pub fn clear(&mut self) {
        self.resize(self.megabytes);
    }

    // Provides hash usage in permille (1 per 1000, as oppposed to percent,
    // which is 1 per 100.)
    pub fn hash_full(&self) -> u16 {
        if self.megabytes > 0 {
            ((self.used_entries * 1000) / self.total_entries) as u16
        } else {
            0
        }
    }
}

// Private functions
impl<D: IHashData + Copy + Clone> HashTable<D> {
    // Calculate the index (bucket) where the data is going to be stored.
    // Use half of the Zobrist key for this.
    fn calculate_index(&self, zobrist_key: ZobristKey) -> usize {
        ((zobrist_key as usize & HIGH_FOUR_BYTES) >> SHIFT_TO_LOWER) % self.total_buckets
    }

    // Many positions will end up at the same index, and thus in the same
    // bucket. Calculate a verification for the position so it can later be
    // found in the bucket. Use the other half of the Zobrist key for this.
    fn calculate_verification(&self, zobrist_key: ZobristKey) -> u32 {
        ((zobrist_key as usize) & LOW_FOUR_BYTES) as u32
    }

    // This function calculates the values for total_buckets and
    // total_entries. These depend on the requested hash size.
    fn calculate_init_values(megabytes: usize) -> (usize, usize) {
        let entry_size = std::mem::size_of::<Entry<D>>();
        let bucket_size = entry_size * ENTRIES_PER_BUCKET;
        let total_buckets = MEGABYTE / bucket_size * megabytes;
        let total_entries = total_buckets * ENTRIES_PER_BUCKET;

        (total_buckets, total_entries)
    }
}
