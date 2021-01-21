/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
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

use super::gamestate::GameState;
use crate::defs::MAX_GAME_MOVES;

// The history struct is basically an array holding the values of the game
// states at each move. If a move is made in make(), this function pushes the
// current game state into this array. In unmake(), that game state can then be
// popped and restored. It is faster than a vector, because:
//
// - It is stored on the stack (a vector is stored on the heap)
// - It doesn't do any error checking. It is up to the caller to check if the
//   history array is either full or empty, before pushing or popping (if
//   necessary, such as during console play: the chess engine will always have
//   one push for every pop during search.)

#[derive(Clone)]
pub struct History {
    list: [GameState; MAX_GAME_MOVES as usize],
    count: usize,
}

impl History {
    // Create a new history array containing game states.
    pub fn new() -> Self {
        Self {
            list: [GameState::new(); MAX_GAME_MOVES as usize],
            count: 0,
        }
    }

    // Wipe the entire array.
    pub fn clear(&mut self) {
        self.list = [GameState::new(); MAX_GAME_MOVES as usize];
        self.count = 0;
    }

    // Put a new game state into the array.
    pub fn push(&mut self, g: GameState) {
        self.list[self.count] = g;
        self.count += 1;
    }

    // Return the last game state and decremnt the counter. The game state is
    // not deleted from the array. If necessary, another game state will just
    // overwrite it.
    pub fn pop(&mut self) -> GameState {
        self.count -= 1;
        self.list[self.count]
    }

    pub fn get_ref(&self, index: usize) -> &GameState {
        &self.list[index]
    }

    pub fn len(&self) -> usize {
        self.count
    }
}
