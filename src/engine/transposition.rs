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
use std::ops::Deref;
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::sync::Arc;
use parking_lot::{RwLock, Mutex};
use by_address::ByAddress;
use concurrent_map::ConcurrentMap;
use explicit_cast::{Truncate, TruncateFrom};
use smallvec::{smallvec, SmallVec};
use thread_local::ThreadLocal;
use crate::{board::defs::ZobristKey, movegen::defs::ShortMove, search::defs::CHECKMATE_THRESHOLD};
use crate::board::Board;

const MEGABYTE: usize = 1024 * 1024;
const ENTRIES_PER_BUCKET: usize = 4;
const BUCKETS_FOR_PARTIAL_HASH: usize = 1 << 32;

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

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum HashFlag {
    Nothing,
    Exact,
    Alpha,
    Beta,
}

#[derive(Copy, Clone, Eq, PartialEq)]
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

#[derive(Copy, Clone, Eq, PartialEq)]
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

#[derive(Clone, Eq, PartialEq)]
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
#[derive(Eq, PartialEq, Clone)]
enum TTCore<D> {
    FullHash(SmallVec<[RehashableBucket<D>; 1]>),
    HalfHash(Vec<NonRehashableBucket<D>>),
}

impl<D> TTCore<D> {
    pub(crate) fn len(&self) -> usize {
        match self {
            TTCore::FullHash(ref tt) => tt.len(),
            TTCore::HalfHash(ref tt) => tt.len(),
        }
    }

    pub(crate) fn size_bytes(&self) -> usize {
        size_of::<Self>() + match self {
            TTCore::FullHash(ref tt) => tt.len() * std::mem::size_of::<RehashableBucket<D>>(),
            TTCore::HalfHash(ref tt) => tt.len() * std::mem::size_of::<NonRehashableBucket<D>>(),
        }
    }

    // Calculate the index (bucket) where the data is going to be stored.
    // Use only the upper half of the Zobrist key for this, so the lower
    // half can be used to calculate a verification.
    fn calculate_index(&self, zobrist_key: ZobristKey) -> usize {
        zobrist_key as usize % self.len()
    }
}

#[derive(Eq, PartialEq, Clone)]
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
        if buckets >= BUCKETS_FOR_PARTIAL_HASH {
            Self {
                tt: TTCore::FullHash(smallvec![RehashableBucket::<D>::new(); buckets]),
                used_entries: 0
            }
        } else {
            Self {
                tt: TTCore::HalfHash(vec![NonRehashableBucket::<D>::new(); buckets]),
                used_entries: 0
            }
        }
    }

    // Resizes the TT by replacing the current TT with a
    // new one. (We don't use Vec's resize function, because it clones
    // elements. This can be problematic if TT sizes push the
    // computer's memory limits.)
    pub fn resize(&mut self, megabytes: usize, room_to_grow: &AtomicIsize) {
        let total_buckets = TT::<D>::calculate_init_buckets(megabytes);

        self.resize_to_bucket_count(total_buckets, room_to_grow);
    }

    fn resize_to_bucket_count(&mut self, buckets: usize, room_to_grow: &AtomicIsize) -> bool {
        let old_bucket_count = self.tt.len();
        let old_size_bytes = self.tt.size_bytes();
        let new_size_bytes = size_of::<TTCore<D>>() + if buckets >= BUCKETS_FOR_PARTIAL_HASH {
            buckets * std::mem::size_of::<RehashableBucket<D>>()
        } else {
            buckets * std::mem::size_of::<NonRehashableBucket<D>>()
        };
        if new_size_bytes > old_size_bytes {
            if room_to_grow.fetch_sub((new_size_bytes - old_size_bytes) as isize, Ordering::AcqRel) < 0 {
                room_to_grow.fetch_add((new_size_bytes - old_size_bytes) as isize, Ordering::AcqRel);
                return false;
            }
        } else {
            room_to_grow.fetch_add((old_size_bytes - new_size_bytes) as isize, Ordering::AcqRel);
        }
        if buckets >= BUCKETS_FOR_PARTIAL_HASH {
            self.tt = TTCore::HalfHash(vec![NonRehashableBucket::<D>::new(); buckets]);
            self.used_entries = 0;
        } else if buckets > old_bucket_count {
            if let TTCore::FullHash(ref mut tt) = self.tt {
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
                return true;
            }
        }
        self.tt = TTCore::FullHash(smallvec![RehashableBucket::<D>::new(); buckets]);
        self.used_entries = 0;
        return true;
    }

    // Insert a position at the calculated index, by storing it in the
    // index's bucket.
    pub fn insert(&mut self, zobrist_key: ZobristKey, data: D, room_to_grow: &AtomicIsize) {
        if self.tt.len() > 0 {
            let verification = self.calculate_verification(zobrist_key);
            while let TTCore::FullHash(ref mut tt) = self.tt {
                let index = zobrist_key as usize % tt.len();
                if !tt[index].store(verification, data, &mut self.used_entries, false) {
                    if !self.resize_to_bucket_count(self.tt.len() * 4, room_to_grow) {
                        if !self.resize_to_bucket_count(self.tt.len() * 2, room_to_grow) {
                            break;
                        }
                    }
                } else {
                    return;
                }
            }
            let index = zobrist_key as usize % self.tt.len();
            match self.tt {
                TTCore::FullHash(ref mut tt) => tt[index].store(verification, data, &mut self.used_entries, true),
                TTCore::HalfHash(ref mut tt) => tt[index].store(verification, data, &mut self.used_entries, true),
            };
        }
    }

    // Probe the TT by both verification and depth. Both have to
    // match for the position to be the correct one we're looking for.
    pub fn probe(&self, zobrist_key: ZobristKey) -> Option<&D> {
        if self.tt.len() > 0 {
            let index = self.tt.calculate_index(zobrist_key);
            let verification = self.calculate_verification(zobrist_key);
            match self.tt {
                TTCore::FullHash(ref tt) => tt[index].find(verification),
                TTCore::HalfHash(ref tt) => tt[index].find(verification),
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
                TTCore::FullHash(ref mut tt) => tt[index].find_mut(verification),
                TTCore::HalfHash(ref mut tt) => tt[index].find_mut(verification)
            }
        } else {
            None
        }
    }

    // Clear TT by replacing it with a new one.
    pub fn clear(&mut self) {
        self.tt = TTCore::FullHash(smallvec![RehashableBucket::<D>::new()]);
        self.used_entries = 0;
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

type OurMap = ConcurrentMap<u32, ByAddress<Arc<RwLock<TT<SearchData>>>>>;

pub struct TTree {
    map: Mutex<OurMap>,
    tts: Arc<ThreadLocal<OurMap>>,
    max_size: AtomicUsize,
    room_to_grow: AtomicIsize
}

impl TTree {
    pub fn new(size_mb: usize) -> Self {
        let size_bytes = size_mb * MEGABYTE;
        Self {
            tts: Arc::new(ThreadLocal::new()),
            map: Mutex::new(ConcurrentMap::new()),
            max_size: AtomicUsize::new(size_bytes),
            room_to_grow: AtomicIsize::new(size_bytes as isize)
        }
    }

    fn get_map(&self) -> &OurMap {
        self.tts.get_or(|| self.map.lock().to_owned())
    }

    pub fn insert(&self, board: &Board, value: SearchData) {
        let zobrist_key = board.game_state.zobrist_key;
        let new_table_size = AtomicUsize::new(0);
        let entry = self.get_map().update_and_fetch(board.monotonic_hash(), |entry| {
            match entry {
                Some(ref e) => {
                    new_table_size.store(0, Ordering::Release);
                    Some(ByAddress::from(Arc::clone(e)))
                },
                None => {
                    let mut new_table = TT::new_with_buckets(1);
                    new_table_size.store(new_table.tt.size_bytes(), Ordering::Release);
                    new_table.insert(zobrist_key, value, &self.room_to_grow);
                    Some(ByAddress::from(Arc::new(RwLock::new(new_table))))
                }
            }
        });
        let new_table_size = new_table_size.into_inner();
        if new_table_size == 0 {
            entry.unwrap().write().insert(zobrist_key, value, &self.room_to_grow);
        } else {
            self.room_to_grow.fetch_sub(new_table_size as isize, Ordering::Release);
        }
    }

    pub fn probe(&self, board: &Board) -> Option<SearchData> {
        self.get_map().get(&board.monotonic_hash())?.read().probe(board.game_state.zobrist_key).cloned()
    }

    pub fn hash_full(&self) -> u16 {
        let max_buckets = (self.max_size.load(Ordering::Relaxed) - (self.get_map().len() * size_of::<TT<SearchData>>())) / size_of::<NonRehashableBucket<SearchData>>();
        let total_entries: usize = self.get_map().iter().map(|(_, t)| t.read().deref().used_entries).sum();
        ((total_entries * 1000 + 500) / (max_buckets * ENTRIES_PER_BUCKET)) as u16
    }

    pub fn remove_unreachable(&self, board: &Board) {
        let unreachable = (board.monotonic_hash() + 1)..=u32::MAX;
        let mut bytes_freed: usize = 0;
        while let Some((_, tt)) = self.get_map().pop_last_in_range(unreachable.clone()) {
            bytes_freed += tt.read().tt.size_bytes();
        }
        self.room_to_grow.fetch_add(bytes_freed as isize, Ordering::AcqRel);
    }

    pub fn clear(&self) {
        while self.get_map().pop_last().is_some() {}
        self.room_to_grow.store(self.max_size.load(Ordering::SeqCst) as isize, Ordering::SeqCst);
    }

    pub fn resize(&self, megabytes: usize) {
        let new_max_size = megabytes * MEGABYTE;
        let mut size_change: isize;
        loop {
            let old_max_size = self.max_size.load(Ordering::Acquire);
            size_change = new_max_size as isize - old_max_size as isize;
            if self.max_size.compare_exchange(old_max_size, new_max_size, Ordering::SeqCst, Ordering::Acquire).is_ok() {
                break;
            }
        }
        let mut new_room_to_grow = self.room_to_grow.fetch_add(size_change, Ordering::SeqCst);
        while new_room_to_grow < 0 {
            let max_buckets = self.get_map().iter().map(
                |(_, tt)| tt.read().tt.len()).max().unwrap();
            let new_max_buckets = max_buckets / 2;
            if new_max_buckets < 1 {
                return;
            }
            let mut bytes_freed = 0;
            for (_, tt) in self.get_map().iter() {
                let mut tt = tt.write();
                if tt.tt.len() > new_max_buckets {
                    let old_size = tt.tt.size_bytes();
                    tt.resize_to_bucket_count(new_max_buckets, &self.room_to_grow);
                    bytes_freed += (old_size - tt.tt.size_bytes()) as isize;
                }
            }
            new_room_to_grow += bytes_freed;
        }
    }
}
