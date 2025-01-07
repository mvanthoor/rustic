// Public functions
use crate::board::defs::ZobristKey;
use crate::transposition::{
    bucket::Bucket,
    defs::{HashData, Transposition},
    entry::{Entry, NR_OF_BUCKETS},
};

const MEGABYTE: usize = 1024 * 1024;
const HIGH_FOUR_BYTES: u64 = 0xFF_FF_FF_FF_00_00_00_00;
const LOW_FOUR_BYTES: u64 = 0x00_00_00_00_FF_FF_FF_FF;
const SHIFT_TO_LOWER: u64 = 32;

impl<T> Transposition<T>
where
    T: HashData + Copy + Clone,
{
    // Create a new TT of the requested size, able to hold the data
    // of type D, where D has to implement HashData, and must be cloneable
    // and copyable.
    pub fn new(megabytes: usize) -> Self {
        let (total_entries, total_buckets) = Self::init_values(megabytes);

        Self {
            table: vec![Entry::<T>::new(); total_entries],
            megabytes,
            used_buckets: 0,
            total_entries,
            total_buckets,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.megabytes > 0
    }

    // Resize the TT if the incoming size is different from the current
    // one; otherwise just clear the TT.
    pub fn resize(&mut self, megabytes: usize) {
        if self.megabytes != megabytes {
            let (total_entries, total_buckets) = Transposition::<T>::init_values(megabytes);

            self.table = vec![Entry::<T>::new(); total_entries];
            self.megabytes = megabytes;
            self.used_buckets = 0;
            self.total_entries = total_entries;
            self.total_buckets = total_buckets;
        } else {
            self.clear();
        }
    }

    // Insert a position at the calculated index, by storing it in the
    // index's bucket.
    pub fn insert(&mut self, zobrist_key: ZobristKey, data: T) {
        if self.is_enabled() {
            let index = self.index_from(zobrist_key);
            let verification = self.verification_from(zobrist_key);
            self.table[index].store(verification, data, &mut self.used_buckets);
        }
    }

    // Probe the TT by both verification and depth. Both have to
    // match for the position to be the correct one we're looking for.
    pub fn probe(&self, zobrist_key: ZobristKey) -> Option<&T> {
        if self.is_enabled() {
            let index = self.index_from(zobrist_key);
            let verification = self.verification_from(zobrist_key);

            self.find_entry(index).find_data(verification)
        } else {
            None
        }
    }

    // Clear TT by replacing entries with empty ones.
    pub fn clear(&mut self) {
        for entry in self.table.iter_mut() {
            *entry = Entry::new();
        }
        self.used_buckets = 0;
    }

    // Provides TT usage per mille (1 per 1000, as opposed to percent,
    // which is 1 per 100.)
    pub fn hash_full(&self) -> u16 {
        if !self.is_enabled() {
            return 0;
        }

        let fraction = self.used_buckets as f64 / self.total_buckets as f64;
        let promille = (fraction * 1000f64).floor();

        promille as u16
    }

    pub fn hash_full_percent(&self) -> u16 {
        if !self.is_enabled() {
            return 0;
        }

        let fraction = self.used_buckets as f64 / self.total_buckets as f64;
        let percent = (fraction * 100f64).floor();

        percent as u16
    }
}

// Private functions
impl<T> Transposition<T>
where
    T: HashData + Copy + Clone,
{
    fn find_entry(&self, index: usize) -> &Entry<T> {
        &self.table[index]
    }

    // Calculate the index (bucket) where the data is going to be stored.
    // Use only the upper half of the Zobrist key for this, so the lower
    // half can be used to calculate a verification.
    fn index_from(&self, zobrist_key: ZobristKey) -> usize {
        let key = (zobrist_key & HIGH_FOUR_BYTES) >> SHIFT_TO_LOWER;

        (key % (self.total_entries as u64)) as usize
    }

    // Many positions will end up at the same index, and thus in the same
    // bucket. Calculate a verification for the position so it can later be
    // found in the bucket. Use the other half of the Zobrist key for this.
    fn verification_from(&self, zobrist_key: ZobristKey) -> u32 {
        (zobrist_key & LOW_FOUR_BYTES) as u32
    }

    // This function calculates the values for total_entries and
    // total_buckets. These depend on the requested TT size.
    fn init_values(megabytes: usize) -> (usize, usize) {
        let bucket_size = std::mem::size_of::<Bucket<T>>();
        let entry_size = bucket_size * NR_OF_BUCKETS;
        let total_entries = MEGABYTE / entry_size * megabytes;
        let total_buckets = total_entries * NR_OF_BUCKETS;

        (total_entries, total_buckets)
    }
}
