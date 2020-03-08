/**
 * The movegen.rs module is the part of the engine that generates chess moves to be searched
 * and evaluate later, in the search and evaluation modules of the program. Note that the move
 * generator is pseudo-legal: that means that it generates all possible moves, regardless if
 * they leave the king in check after that move. The reason is twofold:
 *
 * 1. This speeds up move generation, as _MOST_ moves will be legal. Checking every move for
 * legality would greatly slow down the move generation process.
 * 2. The search might decide to focus only on a subset of moves and discard the rest. Those
 * discarded moves will not be executed or evaluated. If legality checking had been done on
 * those moves, that time would have been wasted.
 */
use crate::board::Board;
use crate::defines::{
    Bitboard, Piece, Side, BISHOP, KING, KNIGHT, PAWN, PNONE, QUEEN, RANK_1, RANK_4, RANK_5,
    RANK_8, ROOK, SQUARE_NAME, WHITE,
};
use crate::movegenerator::init::Movements;
use crate::print;
use crate::utils::square_on_rank;

/*
Move format explanation

"data" contains all the move information, starting from LSB:

Field       :   bits     Decimal values
============================================
PIECE       :   3        0-7 (use only 0-5)
FROM SQUARE :   6        0-63
TO SQUARE   :   6        0-63
CAPTURE     :   3        0-7
PROMOTION   :   3        0-7
ENPASSANT   :   1        0-1

Field:      ENPASSANT   PROMOTION   CAPTURE     TO          FROM        PIECE
            1           111         111         111111      111111      111
Shift:      21 bits     18 bits     15 bits     9 bits      3 bits      0 bits
& Value:    0x1 (1)     0x7 (7)     0x7 (7)     0x3F (63)   0x3F (63)   0x7 (7)

Get the TO field from "data" by:
    -- Shift 9 bits Right
    -- AND (&) with 0x3F

Obviously, storing information in "data" is the other way around.PIECE_NAME
Storing the "To" square: Shift LEFT 9 bits, then XOR with "data".
*/

/**
 * "Shift" is an enumeration containing the offsets of the
 * data fields within the u64 integer containing the
 * the information about a move.
 */
enum Shift {
    Piece = 0,
    FromSq = 3,
    ToSq = 9,
    Capture = 15,
    Promotion = 18,
    EnPassant = 21,
}

/** This part defines the movelist, and the move and its functions */
pub const MAX_LEGAL_MOVES: u8 = 255;
pub type MoveList = Vec<Move>;

pub struct Move {
    data: u64,
}

impl Move {
    pub fn piece(&self) -> u8 {
        ((self.data >> Shift::Piece as u64) & 0x7) as u8
    }

    pub fn from(&self) -> u8 {
        ((self.data >> Shift::FromSq as u64) & 0x3F) as u8
    }

    pub fn to(&self) -> u8 {
        ((self.data >> Shift::ToSq as u64) & 0x3F) as u8
    }

    pub fn captured(&self) -> u8 {
        ((self.data >> Shift::Capture as u64) & 0x7) as u8
    }

    pub fn promoted(&self) -> u8 {
        ((self.data >> Shift::Promotion as u64) & 0x7) as u8
    }

    pub fn en_passant(&self) -> bool {
        let ep = ((self.data >> Shift::EnPassant as u64) & 0x1) as u8;
        ep == 1
    }
}

/**
 * This function actually generates the moves, using private functions in this module.
 * It takes the following parameters:
 *
 * board: a reference to the board/position
 * side: the side to move
 * movement: The movement database, which provides all possible piece movements on all squares.
 *          It uses precalculated moves for each piece on each square, so the move
 *          generator does not have to calculate this over and aover again.
 * list: a mutable reference to a list that will contain the moves.
 *
*/
pub fn generate(board: &Board, side: Side, movement: &Movements, list: &mut MoveList) {
    list.clear();
    piece(KING, board, side, movement, list);
    piece(KNIGHT, board, side, movement, list);
    piece(ROOK, board, side, movement, list);
    piece(BISHOP, board, side, movement, list);
    piece(QUEEN, board, side, movement, list);
    pawns(board, side, movement, list);
}

pub fn square_attacked(board: &Board, side: Side, movement: &Movements, square: u8) -> bool {
    println!("Examine square: {}", SQUARE_NAME[square as usize]);
    let pieces = if side == WHITE {
        board.bb_w
    } else {
        board.bb_b
    };
    let occupancy = board.occupancy();
    let bb_rook = movement.get_slider_attacks(ROOK, square, occupancy);
    let bb_bishop = movement.get_slider_attacks(BISHOP, square, occupancy);
    let bb_knight = movement.get_non_slider_attacks(KNIGHT, square);
    let bb_king = movement.get_non_slider_attacks(KING, square);

    (bb_rook & pieces[ROOK] > 0)
        || (bb_rook & pieces[QUEEN] > 0)
        || (bb_bishop & pieces[BISHOP] > 0)
        || (bb_bishop & pieces[QUEEN] > 0)
        || (bb_knight & pieces[KNIGHT] > 0)
        || (bb_king & pieces[KING] > 0)
}

/**
 * Generates moves for pieces.
 * Basically:
 * - It gets the "from" square.
 * - It gets all the targets for the piece from the Movements object.
 * - The piece can move to all squares that do not contain our own pieces.
 * - Add those moves to the move list.
 */
fn piece(piece: Piece, board: &Board, side: Side, movement: &Movements, list: &mut MoveList) {
    let bb_occupancy = board.occupancy();
    let bb_own_pieces = board.bb_pieces[side];
    let mut bb_pieces = board.get_pieces(piece, side);
    while bb_pieces > 0 {
        let from = next(&mut bb_pieces);
        let bb_target = match piece {
            QUEEN | ROOK | BISHOP => movement.get_slider_attacks(piece, from, bb_occupancy),
            KING | KNIGHT => movement.get_non_slider_attacks(piece, from),
            _ => 0,
        };
        let bb_moves = bb_target & !bb_own_pieces;
        add_move(board, piece, side, from, bb_moves, list);
    }
}

/**
 * Pawns are a bit more complicated, because their attacks and moves are different,
 * but also because they have en-passant and promotion capabilities.
 * It works as such:
 * - Get the "from" square for each pawn.
 * - Push the pawn forward one rank.
 * - If the destination square is empty, "one_step" contains a move. Otherwise, it's 0.
 * - Two_step is a pawn moving two steps. It contains a move if:
 *      * One_step also contains a move
 *      * The next rank is empty
 *      * and the next rank is the fourth (from either WHITE or BLACK's point of view).
 * Then the capture moves are generated; same way as the king/knight moves.
 * An en_passant capture is generated, if the en_passant square in the board position is set,
 * and if the pawn currently being investigated has this square as an attack target.
 * Combine all the moves, and add them to the move list. The add_move function will take care
 * of promotions, adding four possible moves (Q, R, B, and N) to the list instead of one move.
 */
fn pawns(board: &Board, side: Side, movement: &Movements, list: &mut MoveList) {
    // Direction is Up or Down depending on white or black
    let direction = if side == WHITE { 8 } else { -8 };
    let bb_opponent_pieces = board.bb_pieces[side ^ 1];
    let bb_empty = !board.occupancy();
    let bb_fourth = if side == WHITE {
        board.bb_ranks[RANK_4]
    } else {
        board.bb_ranks[RANK_5]
    };
    let mut bb_pawns = board.get_pieces(PAWN, side);
    while bb_pawns > 0 {
        let from = next(&mut bb_pawns);
        let bb_push = 1u64 << (from as i8 + direction);
        let bb_one_step = bb_push & bb_empty;
        let bb_two_step = bb_one_step.rotate_left((64 + direction) as u32) & bb_empty & bb_fourth;
        let bb_targets = movement.get_pawn_attacks(side, from);
        let bb_captures = bb_targets & bb_opponent_pieces;
        let bb_ep_capture = if let Some(ep) = board.en_passant {
            bb_targets & (1u64 << ep)
        } else {
            0
        };
        let moves = bb_one_step | bb_two_step | bb_captures | bb_ep_capture;
        add_move(board, PAWN, side, from, moves, list);
    }
}

/**
 * Get the next set bit from a bitboard.
 * This is used to get the square locations of each piece.
 * For example, the PAWNS bitboard could have 8 bits set.
 * This function returns the index (= square) from that bitboard,
 * and then removes the bit. All pieces/squares (whatver is in
 * the bitboard) have been handled when the bitboard becomes 0.
 * */
fn next(bitboard: &mut Bitboard) -> u8 {
    let location = bitboard.trailing_zeros();
    *bitboard ^= 1u64 << location;
    location as u8
}

/** Determine if the move is a capture; this is the case if there's an opponent piece on the
 * target square of the moving piece. If so, return which piece is on the target square. If
 * there is no piece, PNONE (no piece) will will be returned.
 */
fn captured_piece(board: &Board, side: Side, to_square: u8) -> Piece {
    let bb_target_square = 1u64 << (to_square as u64);
    let bb_opponent_pieces = board.bb_pieces[side ^ 1];
    if bb_target_square & bb_opponent_pieces > 0 {
        return board.which_piece(to_square);
    };
    PNONE
}

/** Adds moves and the data belonging to those moves to a move list.
 * This function also takes care of promotions, by adding four moves
 * to the list instead of one; one move for each promotion possibility.
*/
fn add_move(board: &Board, piece: Piece, side: Side, from: u8, to: Bitboard, list: &mut MoveList) {
    let mut bb_to = to;
    let promotion_rank = if side == WHITE {
        RANK_8 as u8
    } else {
        RANK_1 as u8
    };
    let promotion_pieces: [usize; 4] = [QUEEN, ROOK, BISHOP, KNIGHT];

    while bb_to > 0 {
        let to_square = next(&mut bb_to);
        let capture = captured_piece(board, side, to_square);
        let promotion = (piece == PAWN) && square_on_rank(to_square, promotion_rank);
        let ep_square = if let Some(square) = board.en_passant {
            square
        } else {
            0
        };
        let en_passant = (piece == PAWN) && (ep_square != 0) && (to_square == ep_square);
        let move_data = (piece as u64)
            | ((from as u64) << Shift::FromSq as u64)
            | ((to_square as u64) << Shift::ToSq as u64)
            | ((capture as u64) << Shift::Capture as u64)
            | ((en_passant as u64) << Shift::EnPassant as u64);

        if !promotion {
            let m = Move {
                data: move_data | ((PNONE as u64) << Shift::Promotion as u64),
            };
            list.push(m);
        }

        if promotion {
            for piece in promotion_pieces.iter() {
                let m = Move {
                    data: move_data | ((*piece as u64) << Shift::Promotion as u64),
                };
                list.push(m);
            }
        }
    }
}
