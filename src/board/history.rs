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

    // Returns the number of items in the history array. As make/unmake always
    // push/pop the same amount during search, this function is only needed for
    // checking if the array is not empty when taking back moves from the
    // console. Therefore the code can be unused if the console isn't compiled
    // into the engine.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.count
    }
}
