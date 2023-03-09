use crate::{board::gamestate::GameState, defs::MAX_GAME_MOVES};

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
    list: [GameState; MAX_GAME_MOVES],
    count: usize,
}

impl History {
    // Create a new history array containing game states.
    pub fn new() -> Self {
        Self {
            list: [GameState::new(); MAX_GAME_MOVES],
            count: 0,
        }
    }

    // Put a new game state into the array.
    pub fn push(&mut self, g: GameState) {
        self.list[self.count] = g;
        self.count += 1;
    }

    // Return the last game state and decrement the counter. The game state is
    // not deleted from the array. If necessary, another game state will just
    // overwrite it.
    pub fn pop(&mut self) -> Option<GameState> {
        if self.count > 0 {
            self.count -= 1;
            Some(self.list[self.count])
        } else {
            None
        }
    }

    pub fn get_ref(&self, index: usize) -> &GameState {
        &self.list[index]
    }

    pub fn len(&self) -> usize {
        self.count
    }
}
