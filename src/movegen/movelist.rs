// movelist.rs contains the implementation of an array backed by a counter, as a
// very minimal drop-in replacement for a vector. An array on the stack is
// faster than a vector allocated on the heap. Speed is much more important than
// functionality, so only the bare minimum to be able to use the array is
// implemented. There is error checking or bounds checking. If the array is
// mis-addressed due to a bug, the program panics.

use crate::{defs::MAX_LEGAL_MOVES, movegen::defs::Move};
use std::mem::MaybeUninit;

type MoveListArray = [Move; MAX_LEGAL_MOVES as usize];
type MoveListRaw = MaybeUninit<MoveListArray>;

// Movelist struct holden the array and counter.
#[derive(Copy, Clone)]
pub struct MoveList {
    list: MoveListArray,
    count: u8,
}

impl MoveList {
    // This function uses unsafe code to create an uninitialized move list.
    // We do this because initializing the list with zero's (and then
    // overwriting them with actual moves) is a massive performance hit. In
    // the future we could pass in the move generator so the list can
    // immediately be initialized with moves.
    pub fn new() -> Self {
        Self {
            list: unsafe {
                // Create the memory block.
                let block = MoveListRaw::uninit();

                // *** Officially, the initialization should be here. This
                // is now left up to the caller of this function. ***

                // Immediately extract it; which is not best practice, but
                // required to avoid the mentioned performance hit.
                block.assume_init()
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

    // Return the move at the given index. If out of bounds, the program crashes.
    pub fn get_move(&self, index: u8) -> Move {
        self.list[index as usize]
    }

    pub fn get_mut_ref_move(&mut self, index: u8) -> &mut Move {
        &mut self.list[index as usize]
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.list.swap(a, b);
    }
}
