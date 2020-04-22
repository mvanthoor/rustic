/*
Move format explanation

"data" contains all the move information, starting from LSB:

Field       :   bits     Decimal values
============================================
PIECE       :   3        0-7 (use only 0-6)
FROM SQUARE :   6        0-63
TO SQUARE   :   6        0-63
CAPTURE     :   3        0-7 (captured piece)
PROMOTION   :   3        0-7 (piece promoted to)
ENPASSANT   :   1        0-1
DOUBLESTEP  :   1        0-1
CASTLING    :   1        0-1


Field:      CASTLING    DOUBLESTEP  ENPASSANT   PROMOTION   CAPTURE     TO          FROM        PIECE
            1           1           1           111         111         111111      111111      111
Shift:      23 bits     22 bits     21 bits     18 bits     15 bits     9 bits      3 bits      0 bits
& Value:    0x1         0x1 (1)     0x1 (1)     0x7 (7)     0x7 (7)     0x3F (63)   0x3F (63)   0x7 (7)

Get the TO field from "data" by:
    -- Shift 9 bits Right
    -- AND (&) with 0x3F

Obviously, storing information in "data" is the other way around.PIECE_NAME
Storing the "To" square: Shift LEFT 9 bits, then XOR with "data".
*/

/* "Shift" is an enum which contains the number of bits that needed to be shifted to store
 * move data in a specific place within the u64 integer. This makes sure that, should the
 * format change, the location needs to be changed only within the integer. */
pub enum Shift {
    Piece = 0,
    FromSq = 3,
    ToSq = 9,
    Capture = 15,
    Promotion = 18,
    EnPassant = 21,
    DoubleStep = 22,
    Castling = 23,
}

/* This struct contains the move data. It's a struct so it can be instantiated, and then
 * it can provide all of the methods associated with it to easily decode the move data. */
#[derive(Copy, Clone)]
pub struct Move {
    pub data: u64,
}

// These functions decode the move data.
impl Move {
    pub fn piece(self) -> u8 {
        ((self.data >> Shift::Piece as u64) & 0x7) as u8
    }

    pub fn from(self) -> u8 {
        ((self.data >> Shift::FromSq as u64) & 0x3F) as u8
    }

    pub fn to(self) -> u8 {
        ((self.data >> Shift::ToSq as u64) & 0x3F) as u8
    }

    pub fn captured(self) -> u8 {
        ((self.data >> Shift::Capture as u64) & 0x7) as u8
    }

    pub fn promoted(self) -> u8 {
        ((self.data >> Shift::Promotion as u64) & 0x7) as u8
    }

    pub fn en_passant(self) -> bool {
        ((self.data >> Shift::EnPassant as u64) & 0x1) as u8 == 1
    }

    pub fn double_step(self) -> bool {
        ((self.data >> Shift::DoubleStep as u64) & 0x1) as u8 == 1
    }

    pub fn castling(self) -> bool {
        ((self.data >> Shift::Castling as u64) & 0x1) as u8 == 1
    }
}
