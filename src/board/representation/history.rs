use super::gamestate::GameState;

// TODO: Update comments
const MAX_GAME_MOVES: usize = 2048;
#[derive(Clone)]
pub struct History {
    list: [GameState; MAX_GAME_MOVES as usize],
    count: usize,
}

impl History {
    pub fn new() -> History {
        History {
            list: [GameState::new(); MAX_GAME_MOVES as usize],
            count: 0,
        }
    }

    pub fn clear(&mut self) {
        self.list = [GameState::new(); MAX_GAME_MOVES as usize];
        self.count = 0;
    }

    pub fn push(&mut self, g: GameState) {
        assert!(self.count < MAX_GAME_MOVES, "History list already full.");
        self.list[self.count] = g;
        self.count += 1;
    }

    pub fn pop(&mut self) -> GameState {
        assert!(self.count >= 1, "History list already empty.");
        self.count -= 1;
        self.list[self.count]
    }
}
