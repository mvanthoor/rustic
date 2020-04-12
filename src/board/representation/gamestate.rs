use crate::movegen::movedefs::Move;

// TODO: Update comments

#[derive(Clone, Copy)]
pub struct GameState {
    pub active_color: u8,
    pub castling: u8,
    pub en_passant: Option<u8>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    pub zobrist_key: u64,
    pub this_move: Move,
    pub material: [u16; 2],
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            material: [0; 2],
            active_color: 0,
            castling: 0,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 0,
            zobrist_key: 0,
            this_move: Move { data: 0 },
        }
    }
}
