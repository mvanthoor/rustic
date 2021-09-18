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

use crate::{board::defs::ZobristKey, movegen::defs::ShortMove, search::defs::CHECKMATE_THRESHOLD};

const MEGABYTE: usize = 1024 * 1024;
const NR_OF_BUCKETS: usize = 4;
const HIGH_FOUR_BYTES: u64 = 0xFF_FF_FF_FF_00_00_00_00;
const LOW_FOUR_BYTES: u64 = 0x00_00_00_00_FF_FF_FF_FF;
const SHIFT_TO_LOWER: u64 = 32;

/* ===== Data ========================================================= */

pub trait IHashData {
    fn new() -> Self;
    fn depth(&self) -> i8;
}
#[derive(Copy, Clone)]
pub struct PerftData {
    depth: i8,
    leaf_nodes: u64,
}

impl IHashData for PerftData {
    fn new() -> Self {
        Self {
            depth: 0,
            leaf_nodes: 0,
        }
    }

    fn depth(&self) -> i8 {
        self.depth
    }
}

impl PerftData {
    pub fn create(depth: i8, leaf_nodes: u64) -> Self {
        Self { depth, leaf_nodes }
    }

    pub fn get(&self, depth: i8) -> Option<u64> {
        if self.depth == depth {
            Some(self.leaf_nodes)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub enum HashFlag {
    Nothing,
    Exact,
    Alpha,
    Beta,
}

#[derive(Copy, Clone)]
pub struct SearchData {
    depth: i8,
    flag: HashFlag,
    value: i16,
    best_move: ShortMove,
}

impl IHashData for SearchData {
    fn new() -> Self {
        Self {
            depth: 0,
            flag: HashFlag::Nothing,
            value: 0,
            best_move: ShortMove::new(0),
        }
    }

    fn depth(&self) -> i8 {
        self.depth
    }
}

impl SearchData {
    pub fn create(depth: i8, ply: i8, flag: HashFlag, value: i16, best_move: ShortMove) -> Self {
        // This is the value we're going to save into the TT.
        let mut v = value;

        // If we're dealing with checkmate, the value must be adjusted, so
        // they take the number of plies at which they were found into
        // account, before storing the value into the TT. These ifs can be
        // rewritten as a comparative match expression. We don't, because
        // they're slower. (No inlining by the compiler.)
        if v > CHECKMATE_THRESHOLD {
            v += ply as i16;
        }

        if v < CHECKMATE_THRESHOLD {
            v -= ply as i16;
        }

        Self {
            depth,
            flag,
            value: v,
            best_move,
        }
    }

    pub fn get(&self, depth: i8, ply: i8, alpha: i16, beta: i16) -> (Option<i16>, ShortMove) {
        // We either do, or don't have a value to return from the TT.
        let mut value: Option<i16> = None;

        if self.depth >= depth {
            match self.flag {
                HashFlag::Exact => {
                    // Get the value from the data. We don't want to change
                    // the value that is in the TT.
                    let mut v = self.value;

                    // Adjust for the number of plies from where this data
                    // is probed, if we're dealing with checkmate. Same as
                    // above: no comparative match expression.
                    if v > CHECKMATE_THRESHOLD {
                        v -= ply as i16;
                    }

                    if v < CHECKMATE_THRESHOLD {
                        v += ply as i16;
                    }

                    // This is the value that will be returned.
                    value = Some(v);
                }
                HashFlag::Alpha => {
                    if self.value <= alpha {
                        value = Some(alpha);
                    }
                }
                HashFlag::Beta => {
                    if self.value >= beta {
                        value = Some(beta);
                    }
                }
                _ => (),
            };
        }
        (value, self.best_move)
    }
}

/* ===== Bucket ======================================================== */

#[derive(Copy, Clone)]
struct Bucket<D> {
    verification: u32,
    data: D,
}

impl<D: IHashData> Bucket<D> {
    pub fn new() -> Self {
        Self {
            verification: 0,
            data: D::new(),
        }
    }
}

/* ===== Entry ======================================================= */

#[derive(Clone)]
struct Entry<D> {
    entry: [Bucket<D>; NR_OF_BUCKETS],
}

impl<D: IHashData + Copy> Entry<D> {
    pub fn new() -> Self {
        Self {
            entry: [Bucket::new(); NR_OF_BUCKETS],
        }
    }

    // Store a position in the bucket. Replace the position with the stored
    // lowest depth, as positions with higher depth are more valuable.
    pub fn store(&mut self, verification: u32, data: D, used_buckets: &mut usize) {
        let mut idx_low = 0;

        // Find the index of the entry with the lowest depth.
        for i in 1..NR_OF_BUCKETS {
            if self.entry[i].data.depth() < data.depth() {
                idx_low = i
            }
        }

        // If the verification was 0, this entry in the bucket was never
        // used before. Count the use of this entry.
        if self.entry[idx_low].verification == 0 {
            *used_buckets += 1;
        }

        // Store.
        self.entry[idx_low] = Bucket { verification, data }
    }

    // Find a position in the bucket, where both the stored verification and
    // depth match the requested verification and depth.
    pub fn find(&self, verification: u32) -> Option<&D> {
        for e in self.entry.iter() {
            if e.verification == verification {
                return Some(&e.data);
            }
        }
        None
    }
}

/* ===== TT =================================================== */

// Transposition Table
pub struct TT<D> {
    tt: Vec<Entry<D>>,
    megabytes: usize,
    used_buckets: usize,
    total_buckets: usize,
    total_entries: usize,
}

// Public functions
impl<D: IHashData + Copy + Clone> TT<D> {
    // Create a new TT of the requested size, able to hold the data
    // of type D, where D has to implement IHashData, and must be cloneable
    // and copyable.
    pub fn new(megabytes: usize) -> Self {
        let (total_buckets, total_entries) = Self::calculate_init_values(megabytes);

        Self {
            tt: vec![Entry::<D>::new(); total_buckets],
            megabytes,
            used_buckets: 0,
            total_buckets,
            total_entries,
        }
    }

    // Resizes the TT by replacing the current TT with a
    // new one. (We don't use Vec's resize function, because it clones
    // elements. This can be problematic if TT sizes push the
    // computer's memory limits.)
    pub fn resize(&mut self, megabytes: usize) {
        let (total_buckets, total_entries) = TT::<D>::calculate_init_values(megabytes);

        self.tt = vec![Entry::<D>::new(); total_buckets];
        self.megabytes = megabytes;
        self.used_buckets = 0;
        self.total_buckets = total_buckets;
        self.total_entries = total_entries;
    }

    // Insert a position at the calculated index, by storing it in the
    // index's bucket.
    pub fn insert(&mut self, zobrist_key: ZobristKey, data: D) {
        if self.megabytes > 0 {
            let index = self.calculate_index(zobrist_key);
            let verification = self.calculate_verification(zobrist_key);
            self.tt[index].store(verification, data, &mut self.used_buckets);
        }
    }

    // Probe the TT by both verification and depth. Both have to
    // match for the position to be the correct one we're looking for.
    pub fn probe(&self, zobrist_key: ZobristKey) -> Option<&D> {
        if self.megabytes > 0 {
            let index = self.calculate_index(zobrist_key);
            let verification = self.calculate_verification(zobrist_key);

            self.tt[index].find(verification)
        } else {
            None
        }
    }

    // Clear TT by replacing it with a new one.
    pub fn clear(&mut self) {
        for bucket in self.tt.iter_mut() {
            *bucket = Entry::new();
        }
        self.used_buckets = 0;
    }

    // Provides TT usage in permille (1 per 1000, as opposed to percent,
    // which is 1 per 100.)
    pub fn hash_full(&self) -> u16 {
        if self.megabytes > 0 {
            ((self.used_buckets as f64 / self.total_entries as f64) * 1000f64).floor() as u16
        } else {
            0
        }
    }
}

// Private functions
impl<D: IHashData + Copy + Clone> TT<D> {
    // Calculate the index (bucket) where the data is going to be stored.
    // Use only the upper half of the Zobrist key for this, so the lower
    // half can be used to calculate a verification.
    fn calculate_index(&self, zobrist_key: ZobristKey) -> usize {
        let key = (zobrist_key & HIGH_FOUR_BYTES) >> SHIFT_TO_LOWER;
        let total = self.total_buckets as u64;

        (key % total) as usize
    }

    // Many positions will end up at the same index, and thus in the same
    // bucket. Calculate a verification for the position so it can later be
    // found in the bucket. Use the other half of the Zobrist key for this.
    fn calculate_verification(&self, zobrist_key: ZobristKey) -> u32 {
        (zobrist_key & LOW_FOUR_BYTES) as u32
    }

    // This function calculates the values for total_buckets and
    // total_entries. These depend on the requested TT size.
    fn calculate_init_values(megabytes: usize) -> (usize, usize) {
        let entry_size = std::mem::size_of::<Bucket<D>>();
        let bucket_size = entry_size * NR_OF_BUCKETS;
        let total_buckets = MEGABYTE / bucket_size * megabytes;
        let total_entries = total_buckets * NR_OF_BUCKETS;

        (total_buckets, total_entries)
    }
}
