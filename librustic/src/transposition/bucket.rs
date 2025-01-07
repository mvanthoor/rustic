use crate::transposition::defs::HashData;

#[derive(Copy, Clone)]
pub struct Bucket<D> {
    pub verification: u32,
    pub data: D,
}

impl<D> Bucket<D>
where
    D: HashData,
{
    pub fn new() -> Self {
        Self {
            verification: 0,
            data: D::empty(),
        }
    }
}
