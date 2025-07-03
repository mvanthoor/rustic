use super::defs::Move;
use crate::defs::MAX_LEGAL_MOVES;
use std::mem::{self, MaybeUninit};

type MoveListArray = [Move; MAX_LEGAL_MOVES as usize];
type MoveListRaw = MaybeUninit<MoveListArray>;

// Movelist struct holden the array and counter.
#[derive(Copy, Clone)]
pub struct MoveList {
    list: MoveListArray,
    count: u8,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveList {
    // Creates a new move list. YES, I know that the use of MaybeUninit
    // directly followed by assume_init() is, officially speaking,
    // incorrect because it DOES create a memory block with uninitialized
    // variables. The memory doesn't need to be initialized, because the
    // next step after creating the move list will always be to generate
    // moves and store them in the list beginning at index 0. This would
    // overwrite the initialization and make it useless. Initializing the
    // move list with 0's massively slows down the program. Maybe in the
    // future, I'll rewrite the move generator function to create and fill
    // in the list by itself, without taking a reference to an empty list.
    pub fn new() -> Self {
        Self {
            list: unsafe {
                let raw = mem::MaybeUninit::<MoveListArray>::uninit();
                mem::transmute::<MoveListRaw, MoveListArray>(raw)
            },
            count: 0,
        }
    }

    // Used to store a move in the move list.
    pub fn push(&mut self, m: Move) {
        self.list[self.count as usize] = m;
        self.count += 1;
    }

    // Returns the number of moves in the move list.
    pub fn len(&self) -> u8 {
        self.count
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    // Return the move at the given index. If out of bounds, the program crashes.
    pub fn get_move(&self, index: u8) -> Move {
        self.list[index as usize]
    }

    pub fn get_mut_move(&mut self, index: u8) -> &mut Move {
        &mut self.list[index as usize]
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        unsafe {
            // Take two raw pointers to the moves.
            let ptr_a: *mut Move = &mut self.list[a];
            let ptr_b: *mut Move = &mut self.list[b];

            // Swap those pointers, so now the moves are swapped.
            std::ptr::swap(ptr_a, ptr_b);
        }
    }
}
