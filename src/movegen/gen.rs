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
use super::info;
use super::movedefs::{Move, MoveList, Shift};
use crate::defs::{
    Bitboard, Piece, B1, B8, BISHOP, BLACK, C1, C8, CASTLE_BK, CASTLE_BQ, CASTLE_WK, CASTLE_WQ, D1,
    D8, E1, E8, F1, F8, G1, G8, KING, KNIGHT, PAWN, PNONE, QUEEN, RANK_1, RANK_4, RANK_5, RANK_8,
    ROOK, WHITE,
};
use crate::utils::bits;
use crate::{board, board::representation::Board};

const PROMOTION_PIECES: [usize; 4] = [QUEEN, ROOK, BISHOP, KNIGHT];

/**
 * This function actually generates the moves, using other functions in this module.
 * It takes the following parameters:
 *
 * board: a reference to the board/position
 * side: the side to move
 * mg: The movegenerator, which provides all possible piece attacks on all squares.
 *          It uses precalculated moves for each piece on each square, so the move
 *          generator does not have to calculate this over and aover again.
 * list: a mutable reference to a list that will contain the moves.
*/
pub fn all_moves(board: &Board, list: &mut MoveList) {
    list.clear();
    piece(KING, board, list);
    piece(KNIGHT, board, list);
    piece(ROOK, board, list);
    piece(BISHOP, board, list);
    piece(QUEEN, board, list);
    pawns(board, list);
    castling(board, list);
}

/**
 * Generates moves for pieces.
 * Basically:
 * - It gets the "from" square.
 * - It gets all the targets for the piece from the Movements object.
 * - The piece can move to all squares that do not contain our own pieces.
 * - Add those moves to the move list.
 */
fn piece(piece: Piece, board: &Board, list: &mut MoveList) {
    let side = board.active_color as usize;
    let bb_occupancy = board.occupancy();
    let bb_own_pieces = board.bb_pieces[side];
    let mut bb_pieces = board.get_pieces(piece, side);
    while bb_pieces > 0 {
        let from = bits::next(&mut bb_pieces);
        let bb_target = match piece {
            QUEEN | ROOK | BISHOP => board.get_slider_attacks(piece, from, bb_occupancy),
            KING | KNIGHT => board.get_non_slider_attacks(piece, from),
            _ => 0,
        };
        let bb_moves = bb_target & !bb_own_pieces;
        add_move(board, piece, from, bb_moves, list);
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
fn pawns(board: &Board, list: &mut MoveList) {
    let side = board.active_color as usize;
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
        let from = bits::next(&mut bb_pawns);
        let bb_push = 1u64 << (from as i8 + direction);
        let bb_one_step = bb_push & bb_empty;
        let bb_two_step = bb_one_step.rotate_left((64 + direction) as u32) & bb_empty & bb_fourth;
        let bb_targets = board.get_pawn_attacks(side, from);
        let bb_captures = bb_targets & bb_opponent_pieces;
        let bb_ep_capture = if let Some(ep) = board.en_passant {
            bb_targets & (1u64 << ep)
        } else {
            0
        };
        let moves = bb_one_step | bb_two_step | bb_captures | bb_ep_capture;
        add_move(board, PAWN, from, moves, list);
    }
}

// TODO: Fix this comment
/** The castling function is long, but fortunately not hard to understand.
 * The length is due to having four parts; each side can castle either kingside or queenside.
 * Step by step description:
 * First, determine the opponent, which is "not our side".
 * "has_castling_rights" is checked against the board, for either white or black.
 * "in_check" is either checked for white or black.
 * Then there are two big parts: one for white castling, and one for black castling.
 * A part can be executed, if the side is correct for that part, the side has at least one
 * castling right, and the king is not in check.
 * Inside the part, we try to either castle kingside or queenside. To be able to determine
 * if castling is possible, we first determine if there are any blocking pieces between the
 * king and the rook of the side we're castling to. We also check if the square directly next
 * to the king is not attacked; it's not permitted to castle across check. If there are no
 * blockers and the squares just next to the king are not attacked, castling is possible.
 * Note: we MUST verify if the king does not castle ACROSS check. We DON'T verify if the king
 * castles INTO check on the landing square (gi, c1, g8 or c8). This verification is left up
 * to makemove/unmake move outside of the move generator.
 */
fn castling(board: &Board, list: &mut MoveList) {
    let side = board.active_color as usize;
    let opponent = side ^ 1;
    let has_castling_rights = if side == WHITE {
        (board.castling & (CASTLE_WK + CASTLE_WQ)) > 0
    } else {
        (board.castling & (CASTLE_BK + CASTLE_BQ)) > 0
    };
    let mut bb_king = board.get_pieces(KING, side);
    let from = bits::next(&mut bb_king);
    let bb_occupancy = board.occupancy();

    if side == WHITE && has_castling_rights {
        // Kingside
        if board.castling & CASTLE_WK > 0 {
            let bb_kingside_blockers: u64 = (1u64 << F1) | (1u64 << G1);
            let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

            if !is_kingside_blocked
                && !info::square_attacked(board, opponent, E1)
                && !info::square_attacked(board, opponent, F1)
            {
                let to = (1u64 << from) << 2;
                add_move(board, KING, from, to, list);
            }
        }

        // Queenside
        if board.castling & CASTLE_WQ > 0 {
            let bb_queenside_blockers: u64 = (1u64 << B1) | (1u64 << C1) | (1u64 << D1);
            let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

            if !is_queenside_blocked
                && !info::square_attacked(board, opponent, E1)
                && !info::square_attacked(board, opponent, D1)
            {
                let to = (1u64 << from) >> 2;
                add_move(board, KING, from, to, list);
            }
        }
    }

    if side == BLACK && has_castling_rights {
        // Kingside
        if board.castling & CASTLE_BK > 0 {
            let bb_kingside_blockers: u64 = (1u64 << F8) | (1u64 << G8);
            let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

            if !is_kingside_blocked
                && !info::square_attacked(board, opponent, E8)
                && !info::square_attacked(board, opponent, F8)
            {
                let to = (1u64 << from) << 2;
                add_move(board, KING, from, to, list);
            }
        }

        // Queenside
        if board.castling & CASTLE_BQ > 0 {
            let bb_queenside_blockers: u64 = (1u64 << B8) | (1u64 << C8) | (1u64 << D8);
            let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

            if !is_queenside_blocked
                && !info::square_attacked(board, opponent, E8)
                && !info::square_attacked(board, opponent, D8)
            {
                let to = (1u64 << from) >> 2;
                add_move(board, KING, from, to, list);
            }
        }
    }
}

/** Adds moves and the data belonging to those moves to a move list.
 * This function also takes care of promotions, by adding four moves
 * to the list instead of one; one move for each promotion possibility.
*/
fn add_move(board: &Board, piece: Piece, from: u8, to: Bitboard, list: &mut MoveList) {
    let mut bb_to = to;
    let side = board.active_color as usize;
    let promotion_rank = (if side == WHITE { RANK_8 } else { RANK_1 }) as u8;

    while bb_to > 0 {
        let to_square = bits::next(&mut bb_to);
        let capture = board.piece_list[to_square as usize];
        let promotion = (piece == PAWN) && board::square_on_rank(to_square, promotion_rank);
        let en_passant = if let Some(square) = board.en_passant {
            (piece == PAWN) && (square == to_square)
        } else {
            false
        };
        let double_step = (piece == PAWN) && ((to_square as i8 - from as i8).abs() == 16);
        let castling = (piece == KING) && ((to_square as i8 - from as i8).abs() == 2);
        let move_data = (piece as u64)
            | ((from as u64) << Shift::FromSq as u64)
            | ((to_square as u64) << Shift::ToSq as u64)
            | ((capture as u64) << Shift::Capture as u64)
            | ((en_passant as u64) << Shift::EnPassant as u64)
            | ((double_step as u64) << Shift::DoubleStep as u64)
            | ((castling as u64) << Shift::Castling as u64);

        if !promotion {
            let m = Move {
                data: move_data | ((PNONE as u64) << Shift::Promotion as u64),
            };
            list.push(m);
        } else {
            for piece in PROMOTION_PIECES.iter() {
                let m = Move {
                    data: move_data | ((*piece as u64) << Shift::Promotion as u64),
                };
                list.push(m);
            }
        }
    }
}
