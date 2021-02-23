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

const MEGABYTE: usize = 1024 * 1024;
const ENTRIES_PER_BUCKET: usize = 4;

// ===== Data ==================================================================================//

pub trait IHashData {
    fn new() -> Self;
}
#[derive(Copy, Clone)]
pub struct PerftData {
    pub leaf_nodes: u64,
}

impl IHashData for PerftData {
    fn new() -> Self {
        Self { leaf_nodes: 0 }
    }
}

#[derive(Copy, Clone)]
pub struct SearchData {
    pub stuff1: u32,
    pub stuff2: u32,
}

impl IHashData for SearchData {
    fn new() -> Self {
        Self {
            stuff1: 0,
            stuff2: 0,
        }
    }
}

// ===== Entry ==================================================================================//

#[derive(Copy, Clone)]
struct Entry<D> {
    verification: u32,
    depth: u8,
    data: D,
}

impl<D: IHashData> Entry<D> {
    pub fn new() -> Self {
        Self {
            verification: 0,
            depth: 0,
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
}

// ===== Hash table ============================================================================ //

pub struct HashTable<D> {
    hash_table: Vec<Bucket<D>>,
    megabytes: usize,
    total_entries: usize,
}

impl<D: IHashData + Copy + Clone> HashTable<D> {
    pub fn new(megabytes: usize) -> Self {
        let entry_size = std::mem::size_of::<Entry<D>>();
        let bucket_size = entry_size * ENTRIES_PER_BUCKET;
        let buckets = MEGABYTE / bucket_size * megabytes;
        let total_entries = buckets * ENTRIES_PER_BUCKET;

        Self {
            hash_table: vec![Bucket::<D>::new(); buckets],
            megabytes,
            total_entries,
        }
    }
}
