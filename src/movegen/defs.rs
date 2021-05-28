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

/*
Move format explanation

"data" contains all the move information, starting from LSB:

Field       :   bits     Decimal values
============================================
PIECE       :   3        0-7 (use only 0-6)
FROM        :   6        0-63
TO          :   6        0-63
CAPTURE     :   3        0-7 (captured piece)
PROMOTION   :   3        0-7 (piece promoted to)
ENPASSANT   :   1        0-1
DOUBLESTEP  :   1        0-1
CASTLING    :   1        0-1
SORTSCORE   :   16       0-65536


---------------------------------- move data -------------------------------------------
0000000000000000    0        0          0         000       000     000000 000000 000
SORTSCORE           CASTLING DOUBLESTEP ENPASSANT PROMOTION CAPTURE TO     FROM   PIECE
----------------------------------------------------------------------------------------

Field:      PROMOTION   CAPTURE     TO          FROM        PIECE
Bits:       3           3           6           6           3
Shift:      18 bits     15 bits     9 bits      3 bits      0 bits
& Value:    0x7 (7)     0x7 (7)     0x3F (63)   0x3F (63)   0x7 (7)

Field:      SORTSCORE   CASTLING    DOUBLESTEP  ENPASSANT
Bits:       16          1           1           1
Shift:      24 bits     23 bits     22 bits     21 bits
& Value:    0xFFFF        0x1         0x1 (1)     0x1 (1)

Get the TO field from "data" by:
    -- Shift 9 bits Right
    -- AND (&) with 0x3F

Obviously, storing information in "data" is the other way around.PIECE_NAME
Storing the "To" square: Shift LEFT 9 bits, then XOR with "data".
*/

pub use super::{magics::Magic, movelist::MoveList};
use crate::{
    board::defs::{PIECE_CHAR_SMALL, SQUARE_NAME},
    defs::{Piece, Square},
};

const MOVE_ONLY: usize = 0x00_00_00_00_00_FF_FF_FF;

/* "Shift" is an enum which contains the number of bits that needed to be shifted to store
 * move data in a specific place within the u64 integer. This makes sure that, should the
 * format change, the location needs to be changed only within the integer. */
pub struct Shift;
impl Shift {
    pub const PIECE: usize = 0;
    pub const FROM_SQ: usize = 3;
    pub const TO_SQ: usize = 9;
    pub const CAPTURE: usize = 15;
    pub const PROMOTION: usize = 18;
    pub const EN_PASSANT: usize = 21;
    pub const DOUBLE_STEP: usize = 22;
    pub const CASTLING: usize = 23;
    pub const SORTSCORE: usize = 24;
}

#[derive(Copy, Clone, PartialEq)]
pub enum MoveType {
    Quiet,
    Capture,
    All,
}

/* This struct contains the move data. It's a struct so it can be instantiated, and then
 * it can provide all of the methods associated with it to easily decode the move data. */
#[derive(Copy, Clone, PartialEq)]
pub struct Move {
    data: usize,
}

// These functions decode the move data.
impl Move {
    pub fn new(data: usize) -> Self {
        Self { data }
    }

    pub fn piece(&self) -> Piece {
        ((self.data >> Shift::PIECE as u64) & 0x7) as Piece
    }

    pub fn from(&self) -> Square {
        ((self.data >> Shift::FROM_SQ as u64) & 0x3F) as Square
    }

    pub fn to(&self) -> Square {
        ((self.data >> Shift::TO_SQ as u64) & 0x3F) as Square
    }

    pub fn captured(&self) -> Piece {
        ((self.data >> Shift::CAPTURE as u64) & 0x7) as Piece
    }

    pub fn promoted(&self) -> Piece {
        ((self.data >> Shift::PROMOTION as u64) & 0x7) as Piece
    }

    pub fn en_passant(&self) -> bool {
        ((self.data >> Shift::EN_PASSANT as u64) & 0x1) as u8 == 1
    }

    pub fn double_step(&self) -> bool {
        ((self.data >> Shift::DOUBLE_STEP as u64) & 0x1) as u8 == 1
    }

    pub fn castling(&self) -> bool {
        ((self.data >> Shift::CASTLING as u64) & 0x1) as u8 == 1
    }

    pub fn sort_score(self) -> u16 {
        ((self.data >> Shift::SORTSCORE as u64) & 0xFFFF) as u16
    }

    pub fn set_score(&mut self, value: u16) {
        self.data |= (value as usize) << Shift::SORTSCORE;
    }

    pub fn as_string(&self) -> String {
        format!(
            "{}{}{}",
            SQUARE_NAME[self.from()],
            SQUARE_NAME[self.to()],
            PIECE_CHAR_SMALL[self.promoted()]
        )
    }

    pub fn to_short_move(&self) -> ShortMove {
        ShortMove::new((self.data & MOVE_ONLY) as u32)
    }

    pub fn get_move(&self) -> u32 {
        (self.data & MOVE_ONLY) as u32
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct ShortMove {
    data: u32,
}

impl ShortMove {
    pub fn new(m: u32) -> Self {
        Self { data: m }
    }

    pub fn get_move(&self) -> u32 {
        self.data
    }
}
