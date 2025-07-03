use crate::transposition::{bucket::Bucket, defs::HashData};

pub const NR_OF_BUCKETS: usize = 3;

#[derive(Clone)]
pub struct Entry<T> {
    entry: [Bucket<T>; NR_OF_BUCKETS],
}

impl<T> Default for Entry<T>
where
    T: HashData + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Entry<T>
where
    T: HashData + Copy,
{
    pub fn new() -> Self {
        Self {
            entry: [Bucket::new(); NR_OF_BUCKETS],
        }
    }

    // Store a position in the bucket. Replace the position with the stored
    // lowest depth, as positions with higher depth are more valuable.
    pub fn store(&mut self, verification: u32, data: T, used_buckets: &mut usize) {
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
    pub fn find_data(&self, verification: u32) -> Option<&T> {
        for bucket in self.entry.iter() {
            if bucket.verification == verification {
                return Some(&bucket.data);
            }
        }
        None
    }
}
