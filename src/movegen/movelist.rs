// TODO: Update comments

use super::movedefs::Move;
use std::mem;

pub const MAX_MOVES: u8 = 255;

#[derive(Copy, Clone)]
pub struct MoveList {
    list: [Move; MAX_MOVES as usize],
    count: u8,
}

impl MoveList {
    pub fn new() -> MoveList {
        MoveList {
            list: unsafe { mem::MaybeUninit::uninit().assume_init() },
            count: 0,
        }
    }

    pub fn push(&mut self, m: Move) {
        self.list[self.count as usize] = m;
        self.count += 1;
    }

    pub fn len(&self) -> u8 {
        self.count
    }

    pub fn get_move(&self, index: u8) -> Move {
        self.list[index as usize]
    }
}
