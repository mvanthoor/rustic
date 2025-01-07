use crate::board::defs::ZobristKey;

const MEGABYTE: usize = 1024 * 1024;
const NR_OF_BUCKETS: usize = 3;
const HIGH_FOUR_BYTES: u64 = 0xFF_FF_FF_FF_00_00_00_00;
const LOW_FOUR_BYTES: u64 = 0x00_00_00_00_FF_FF_FF_FF;
const SHIFT_TO_LOWER: u64 = 32;

/* ===== Data ========================================================= */

pub trait IHashData {
    fn empty() -> Self;
    fn depth(&self) -> i8;
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
            data: D::empty(),
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
            if self.entry[i].data.depth() < self.entry[idx_low].data.depth() {
                idx_low = i
            }
        }

        // If the verification was 0, this entry in the bucket was never
        // used before. Count the use of this entry.
        if self.entry[idx_low].verification == 0 {
            *used_buckets += 1;
        }

        // Store. (Always replace.)
        self.entry[idx_low] = Bucket { verification, data }
    }

    // Find a position in the bucket, where both the stored verification and
    // depth match the requested verification and depth.
    pub fn find(&self, verification: u32) -> Option<&D> {
        for bucket in self.entry.iter() {
            if bucket.verification == verification {
                return Some(&bucket.data);
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
    total_entries: usize,
    total_buckets: usize,
}

// Public functions
impl<D: IHashData + Copy + Clone> TT<D> {
    // Create a new TT of the requested size, able to hold the data
    // of type D, where D has to implement IHashData, and must be cloneable
    // and copyable.
    pub fn new(megabytes: usize) -> Self {
        let (total_entries, total_buckets) = Self::init_values(megabytes);

        Self {
            tt: vec![Entry::<D>::new(); total_entries],
            megabytes,
            used_buckets: 0,
            total_entries,
            total_buckets,
        }
    }

    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.megabytes > 0
    }

    // Resize the TT if the incoming size is different from the current
    // one; otherwise just clear the TT.
    pub fn resize(&mut self, megabytes: usize) {
        if self.megabytes != megabytes {
            let (total_entries, total_buckets) = TT::<D>::init_values(megabytes);

            self.tt = vec![Entry::<D>::new(); total_entries];
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
    pub fn insert(&mut self, zobrist_key: ZobristKey, data: D) {
        if self.megabytes > 0 {
            let index = self.index_from(zobrist_key);
            let verification = self.verification_from(zobrist_key);
            self.tt[index].store(verification, data, &mut self.used_buckets);
        }
    }

    // Probe the TT by both verification and depth. Both have to
    // match for the position to be the correct one we're looking for.
    pub fn probe(&self, zobrist_key: ZobristKey) -> Option<&D> {
        if self.megabytes > 0 {
            let index = self.index_from(zobrist_key);
            let verification = self.verification_from(zobrist_key);

            self.tt[index].find(verification)
        } else {
            None
        }
    }

    // Clear TT by replacing entries with empty ones.
    pub fn clear(&mut self) {
        for entry in self.tt.iter_mut() {
            *entry = Entry::new();
        }
        self.used_buckets = 0;
    }

    // Provides TT usage per mille (1 per 1000, as opposed to percent,
    // which is 1 per 100.)
    pub fn hash_full(&self) -> u16 {
        if self.megabytes == 0 {
            return 0;
        }

        let fraction = self.used_buckets as f64 / self.total_buckets as f64;
        let promille = (fraction * 1000f64).floor();

        promille as u16
    }

    pub fn hash_full_percent(&self) -> u16 {
        if self.megabytes == 0 {
            return 0;
        }

        let fraction = self.used_buckets as f64 / self.total_buckets as f64;
        let percent = (fraction * 100f64).floor();

        percent as u16
    }
}

// Private functions
impl<D: IHashData + Copy + Clone> TT<D> {
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
        let bucket_size = std::mem::size_of::<Bucket<D>>();
        let entry_size = bucket_size * NR_OF_BUCKETS;
        let total_entries = MEGABYTE / entry_size * megabytes;
        let total_buckets = total_entries * NR_OF_BUCKETS;

        (total_entries, total_buckets)
    }
}
