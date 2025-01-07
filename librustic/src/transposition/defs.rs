use crate::transposition::entry::Entry;

pub trait HashData {
    fn empty() -> Self;
    fn depth(&self) -> i8;
}

// Transposition Table
pub struct Transposition<D> {
    pub table: Vec<Entry<D>>,
    pub megabytes: usize,
    pub used_buckets: usize,
    pub total_entries: usize,
    pub total_buckets: usize,
}
