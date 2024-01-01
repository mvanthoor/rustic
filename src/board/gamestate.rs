use crate::{
    board::defs::{Pieces, PIECE_NAME, SQUARE_NAME},
    defs::{Castling, Sides},
    evaluation::defs::W,
    movegen::defs::Move,
};
use std::fmt::{self, Display};

// This is simply a struct that collects all the variables holding the game sate.
// It makes it very easy to make a backup of the game state during make(), and
// restore it when performing unmake(). It prevents having to backup and restore
// each game state variable one by one.

#[derive(Clone, Copy)]
pub struct GameState {
    pub active_color: u8,
    pub castling: u8,
    pub halfmove_clock: u8,
    pub en_passant: Option<u8>,
    pub fullmove_number: u16,
    pub zobrist_key: u64,
    pub phase_value: i16,
    pub psqt_value: [W; Sides::BOTH],
    pub next_move: Move,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            active_color: 0,
            castling: 0,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 0,
            zobrist_key: 0,
            phase_value: 0,
            psqt_value: [W(0, 0); Sides::BOTH],
            next_move: Move::new(0),
        }
    }

    pub fn castling_to_string(&self) -> String {
        let mut s: String = String::from("");
        let c = self.castling;

        s += if c & Castling::WK > 0 { "K" } else { "" };
        s += if c & Castling::WQ > 0 { "Q" } else { "" };
        s += if c & Castling::BK > 0 { "k" } else { "" };
        s += if c & Castling::BQ > 0 { "q" } else { "" };

        if s.is_empty() {
            s = String::from("-");
        }

        s
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ep = if let Some(x) = self.en_passant {
            SQUARE_NAME[x as usize]
        } else {
            "-"
        };

        let promotion = if self.next_move.promoted() != Pieces::NONE {
            PIECE_NAME[self.next_move.promoted()]
        } else {
            ""
        };

        write!(
            f,
            "zk: {:x} ac: {} cperm: {} ep: {} hmc: {} fmn: {}, pst_mg: {}/{}, pst_eg: {}/{} next: {}{}{}",
            self.zobrist_key,
            self.active_color,
            self.castling_to_string(),
            ep,
            self.halfmove_clock,
            self.fullmove_number,
            self.psqt_value[Sides::WHITE].mg(),
            self.psqt_value[Sides::BLACK].mg(),
            self.psqt_value[Sides::WHITE].eg(),
            self.psqt_value[Sides::BLACK].eg(),
            SQUARE_NAME[self.next_move.from()],
            SQUARE_NAME[self.next_move.to()],
            promotion
        )
    }
}
