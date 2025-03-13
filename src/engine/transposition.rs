/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2024, Marcel Vanthoor
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
use std::collections::BTreeMap;
use explicit_cast::{Truncate, TruncateFrom};
use smallvec::{smallvec, SmallVec};
use crate::{board::defs::ZobristKey, movegen::defs::ShortMove, search::defs::CHECKMATE_THRESHOLD};
use crate::board::Board;

const MEGABYTE: usize = 1024 * 1024;
const ENTRIES_PER_BUCKET: usize = 4;
const PRESERVING_RESIZE_LIMIT: usize = 1024;

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
#[repr(u8)]
pub enum HashFlag {
    Nothing,
    Exact,
    Alpha,
    Beta,
}

#[derive(Copy, Clone)]
#[repr(packed)]
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

/* ===== Entry ======================================================== */

#[derive(Copy, Clone)]
struct Entry<V: Copy, D> {
    verification: V,
    data: D,
}

type RehashableEntry<D> = Entry<u64, D>;
type NonRehashableEntry<D> = Entry<u32, D>;

impl <V: Copy + TruncateFrom<u128>, D: IHashData> Entry<V, D> {
    fn new() -> Self {
        Self {
            verification: 0.truncate(),
            data: D::new(),
        }
    }
}

/* ===== Bucket ======================================================= */

#[derive(Clone)]
struct Bucket<E> {
    bucket: [E; ENTRIES_PER_BUCKET],
}

type RehashableBucket<D> = Bucket<RehashableEntry<D>>;
type NonRehashableBucket<D> = Bucket<NonRehashableEntry<D>>;

impl<D: IHashData + Copy, V: Eq + Copy> Bucket<Entry<V, D>> where V: TruncateFrom<u128> {
    fn new() -> Self {
        Self {
            bucket: [Entry::new(); ENTRIES_PER_BUCKET],
        }
    }

    fn store(&mut self, verification: u64, data: D, used_entries: &mut usize, overwrite: bool) -> bool {
        let mut idx_lowest_depth = 0;

        // Find the index of the entry with the lowest depth.
        for entry in 1..ENTRIES_PER_BUCKET {
            if self.bucket[entry].data.depth() < data.depth() {
                idx_lowest_depth = entry
            }
        }

        // If the verifiaction was 0, this entry in the bucket was never
        // used before. Count the use of this entry.
        if self.bucket[idx_lowest_depth].verification == 0.truncate() {
            *used_entries += 1;
        } else if !overwrite {
            // If the entry was used before, and we're not overwriting
            // the entry, return false.
            return false;
        }

        // Store.
        self.bucket[idx_lowest_depth] = Entry { verification: (verification as u128).truncate(), data };
        true
    }

    fn find(&self, verification: u64) -> Option<&D> {
        let verification = (verification as u128).truncate();
        for e in self.bucket.iter() {
            if e.verification == verification {
                return Some(&e.data);
            }
        }
        None
    }

    fn find_mut(&mut self, verification: u64) -> Option<&mut D> {
        let verification = (verification as u128).truncate();
        for e in self.bucket.iter_mut() {
            if e.verification == verification {
                return Some(&mut e.data);
            }
        }
        None
    }
}

/* ===== TT =================================================== */

// Transposition Table

enum TTCore<D> {
    Growable(SmallVec<[RehashableBucket<D>; 1]>),
    NonGrowable(Vec<NonRehashableBucket<D>>),
}

impl<D> TTCore<D> {
    pub(crate) fn len(&self) -> usize {
        match self {
            TTCore::Growable(ref tt) => tt.len(),
            TTCore::NonGrowable(ref tt) => tt.len(),
        }
    }

    // Calculate the index (bucket) where the data is going to be stored.
    // Use only the upper half of the Zobrist key for this, so the lower
    // half can be used to calculate a verification.
    fn calculate_index(&self, zobrist_key: ZobristKey) -> usize {
        zobrist_key as usize % self.len()
    }
}

pub struct TT<D> {
    tt: TTCore<D>,
    used_entries: usize,
}

// Public functions
impl<D: IHashData + Copy + Clone> TT<D> {
    // Create a new TT of the requested size, able to hold the data
    // of type D, where D has to implement IHashData, and must be clonable
    // and copyable.
    pub fn new(megabytes: usize) -> Self {
        let total_buckets = Self::calculate_init_buckets(megabytes);

        Self::new_with_buckets(total_buckets)
    }

    fn new_with_buckets(buckets: usize) -> TT<D> {
        if buckets > PRESERVING_RESIZE_LIMIT {
            Self {
                tt: TTCore::Growable(smallvec![RehashableBucket::<D>::new(); buckets]),
                used_entries: 0
            }
        } else {
            Self {
                tt: TTCore::NonGrowable(vec![NonRehashableBucket::<D>::new(); buckets]),
                used_entries: 0
            }
        }
    }

    // Resizes the TT by replacing the current TT with a
    // new one. (We don't use Vec's resize function, because it clones
    // elements. This can be problematic if TT sizes push the
    // computer's memory limits.)
    pub fn resize(&mut self, megabytes: usize) {
        let total_buckets = TT::<D>::calculate_init_buckets(megabytes);

        self.resize_to_bucket_count(total_buckets);
    }

    fn resize_to_bucket_count(&mut self, buckets: usize) {
        let old_bucket_count = self.tt.len();
        if buckets > PRESERVING_RESIZE_LIMIT {
            self.tt = TTCore::NonGrowable(vec![NonRehashableBucket::<D>::new(); buckets]);
            self.used_entries = 0;
        } else if buckets > old_bucket_count {
            if let TTCore::Growable(ref mut tt) = self.tt {
                tt.resize(buckets, RehashableBucket::<D>::new());
                let (old_buckets, new_buckets) = tt.split_at_mut(old_bucket_count);
                for (index, bucket) in old_buckets.iter_mut().enumerate() {
                    for entry in bucket.bucket.iter_mut() {
                        if entry.verification != 0 {
                            let zobrist_key = entry.verification;
                            let new_index = zobrist_key as usize % buckets;
                            if new_index != index {
                                debug_assert!(new_index > index, "rehashing from bucket {} of {} to bucket {} of {}",
                                              index, old_bucket_count, new_index, buckets);
                                debug_assert!(new_index - old_bucket_count < new_buckets.len(),
                                              "rehashing from bucket {} of {} to bucket {} of {}",
                                              index, old_bucket_count, new_index, buckets);
                                new_buckets[new_index - old_bucket_count].store(entry.verification, entry.data, &mut self.used_entries, false);
                                entry.clone_from(&RehashableEntry::new());
                            }
                        }
                    }
                }
                return;
            }
        }
        self.tt = TTCore::Growable(smallvec![RehashableBucket::<D>::new(); buckets]);
        self.used_entries = 0;
    }

    // Insert a position at the calculated index, by storing it in the
    // index's bucket.
    pub fn insert(&mut self, zobrist_key: ZobristKey, data: D) {
        if self.tt.len() > 0 {
            let verification = self.calculate_verification(zobrist_key);
            while let TTCore::Growable(ref mut tt) = self.tt {
                let index = zobrist_key as usize % tt.len();
                if !tt[index].store(verification, data, &mut self.used_entries, false) {
                    self.resize_to_bucket_count(self.tt.len() * 4);
                } else {
                    return;
                }
            }
            let TTCore::NonGrowable(ref mut tt) = self.tt else {
                unreachable!();
            };
            let index = zobrist_key as usize % tt.len();
            tt[index].store(verification, data, &mut self.used_entries, true);
        }
    }

    // Probe the TT by both verification and depth. Both have to
    // match for the position to be the correct one we're looking for.
    pub fn probe(&self, zobrist_key: ZobristKey) -> Option<&D> {
        if self.tt.len() > 0 {
            let index = self.tt.calculate_index(zobrist_key);
            let verification = self.calculate_verification(zobrist_key);
            match self.tt {
                TTCore::Growable(ref tt) => tt[index].find(verification),
                TTCore::NonGrowable(ref tt) => tt[index].find(verification),
            }
        } else {
            None
        }
    }

    pub fn probe_mut(&mut self, zobrist_key: ZobristKey) -> Option<&mut D> {
        if self.tt.len() > 0 {
            let index = self.tt.calculate_index(zobrist_key);
            let verification = self.calculate_verification(zobrist_key);
            match &mut self.tt{
                TTCore::Growable(ref mut tt) => tt[index].find_mut(verification),
                TTCore::NonGrowable(ref mut tt) => tt[index].find_mut(verification)
            }
        } else {
            None
        }
    }

    // Clear TT by replacing it with a new one.
    pub fn clear(&mut self) {
        self.resize_to_bucket_count(self.tt.len());
    }

    // Provides TT usage in permille (1 per 1000, as oppposed to percent,
    // which is 1 per 100.)
    pub fn hash_full(&self) -> u16 {
        if self.tt.len() > 0 {
            ((self.used_entries as f64 / (self.tt.len() * ENTRIES_PER_BUCKET) as f64) * 1000f64).floor() as u16
        } else {
            0
        }
    }
}

// Private functions
impl<D: IHashData + Copy + Clone> TT<D> {
    // Many positions will end up at the same index, and thus in the same
    // bucket. Calculate a verification for the position so it can later be
    // found in the bucket. Use the other half of the Zobrist key for this.
    fn calculate_verification(&self, zobrist_key: ZobristKey) -> u64 {
        zobrist_key
    }

    // This function calculates the value for total_buckets depending on the
    // requested TT size.
    fn calculate_init_buckets(megabytes: usize) -> usize {
        megabytes * MEGABYTE / size_of::<RehashableBucket<D>>()
    }
}

pub struct TTree(BTreeMap<u32, TT<SearchData>>);
impl TTree {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert(&mut self, board: &Board, value: SearchData) {
        let entry = self.0.entry(board.monotonic_hash()).or_insert_with(|| TT::new_with_buckets(1));
        entry.insert(board.game_state.zobrist_key, value);
    }

    pub fn probe(&self, board: &Board) -> Option<&SearchData> {
        self.0.get(&board.monotonic_hash())?.probe(board.game_state.zobrist_key)
    }

    pub fn probe_mut(&mut self, board: &Board) -> Option<&mut SearchData> {
        self.0.get_mut(&board.monotonic_hash())?.probe_mut(board.game_state.zobrist_key)
    }

    pub fn hash_full(&self) -> u16 {
        let total_buckets: usize = self.0.values().map(|t| t.tt.len()).sum();
        let total_entries: usize = self.0.values().map(|t| t.used_entries).sum();
        ((total_entries * 1000 + 500) / (total_buckets * ENTRIES_PER_BUCKET)) as u16
    }

    pub fn remove_unreachable(&mut self, board: &Board) {
        self.0.split_off(&(board.monotonic_hash() + 1));
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn resize_to_max(&mut self, megabytes: usize) {
        let max_buckets = TT::<SearchData>::calculate_init_buckets(megabytes);
        for tt in self.0.values_mut() {
            if tt.tt.len() > max_buckets {
                tt.resize_to_bucket_count(max_buckets);
            }
        }
    }
}
