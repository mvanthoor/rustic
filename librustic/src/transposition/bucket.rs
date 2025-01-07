use crate::transposition::defs::HashData;

#[derive(Copy, Clone)]
pub struct Bucket<T> {
    pub verification: u32,
    pub data: T,
}

impl<T> Bucket<T>
where
    T: HashData,
{
    pub fn new() -> Self {
        Self {
            verification: 0,
            data: T::empty(),
        }
    }
}
