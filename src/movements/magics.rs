use crate::defines::Bitboard;

#[derive(Copy, Clone)]
pub struct Magics {
    pub mask: Bitboard,
    pub shift: u8,
    pub magic: u64,
    pub offset: u32,
}

impl Default for Magics {
    fn default() -> Magics {
        Magics {
            mask: 0,
            shift: 0,
            magic: 0,
            offset: 0,
        }
    }
}

impl Magics {
    fn index(square: u8) {}
}
