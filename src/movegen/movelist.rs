/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2024, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

// movelist.rs contains the implementation of an array backed by a counter, as a
// very minimal drop-in replacement for a vector. An array on the stack is
// faster than a vector allocated on the heap. Speed is much more important than
// functionality, so only the bare minimum to be able to use the array is
// implemented. There is error checking or bounds checking. If the array is
// mis-addressed due to a bug, the program panics.

use super::defs::Move;
use crate::defs::MAX_LEGAL_MOVES;
use std::mem;

// Movelist struct holden the array and counter.
#[derive(Copy, Clone)]
pub struct MoveList {
    list: [Move; MAX_LEGAL_MOVES as usize],
    count: u8,
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
                let block = mem::MaybeUninit::uninit();
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
