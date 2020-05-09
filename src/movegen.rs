pub mod attackboards;
pub mod blockerboards;
pub mod defs;
pub mod info;
mod init;
pub mod magics;
pub mod masks;
pub mod movelist;
mod rays;

// TODO: Rewrite comments for move generator
use crate::{
    board::{
        defs::{Pieces, Ranks, Squares, BB_RANKS},
        representation::Board,
        utils,
    },
    defs::{Bitboard, Castling, Piece, Side, Square, BLACK, EMPTY, NR_OF_SQUARES, WHITE},
    misc::bits,
};
use defs::{Move, Shift};
use init::{init_king, init_knight, init_magics, init_pawns};
use magics::Magics;
use movelist::MoveList;

const PROMOTION_PIECES: [usize; 4] = [Pieces::QUEEN, Pieces::ROOK, Pieces::BISHOP, Pieces::KNIGHT];

const WHITE_BLACK: usize = 2;
pub const ROOK_TABLE_SIZE: usize = 102_400; // Total permutations of all rook blocker boards.
pub const BISHOP_TABLE_SIZE: usize = 5_248; // Total permutations of all bishop blocker boards.

pub struct MoveGenerator {
    king: [Bitboard; NR_OF_SQUARES],
    knight: [Bitboard; NR_OF_SQUARES],
    pawns: [[Bitboard; NR_OF_SQUARES]; WHITE_BLACK],
    rook: Vec<Bitboard>,
    bishop: Vec<Bitboard>,
    rook_magics: [Magics; NR_OF_SQUARES],
    bishop_magics: [Magics; NR_OF_SQUARES],
}

impl MoveGenerator {
    pub fn new() -> Self {
        let magics: Magics = Default::default();
        let mut mg = Self {
            king: [EMPTY; NR_OF_SQUARES],
            knight: [EMPTY; NR_OF_SQUARES],
            pawns: [[EMPTY; NR_OF_SQUARES]; WHITE_BLACK],
            rook: vec![EMPTY; ROOK_TABLE_SIZE],
            bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            rook_magics: [magics; NR_OF_SQUARES],
            bishop_magics: [magics; NR_OF_SQUARES],
        };
        init_king(&mut mg);
        init_knight(&mut mg);
        init_pawns(&mut mg);
        init_magics(&mut mg, Pieces::ROOK);
        init_magics(&mut mg, Pieces::BISHOP);
        mg
    }

    //** This function takes a board, and generates all moves for the side that is to move. */
    pub fn gen_all_moves(&self, board: &Board, ml: &mut MoveList) {
        self.piece(board, Pieces::KING, ml);
        self.piece(board, Pieces::KNIGHT, ml);
        self.piece(board, Pieces::ROOK, ml);
        self.piece(board, Pieces::BISHOP, ml);
        self.piece(board, Pieces::QUEEN, ml);
        self.pawns(board, ml);
        self.castling(board, ml);
    }

    /** Return non-slider (King, Knight) attacks for the given square. */
    pub fn get_non_slider_attacks(&self, piece: Piece, square: Square) -> Bitboard {
        match piece {
            Pieces::KING => self.king[square],
            Pieces::KNIGHT => self.knight[square],
            _ => panic!("Not a king or a knight: {}", piece),
        }
    }

    /** Return slider attacsk for Rook, Bishop and Queen using Magic. */
    pub fn get_slider_attacks(
        &self,
        piece: Piece,
        square: Square,
        occupancy: Bitboard,
    ) -> Bitboard {
        match piece {
            Pieces::ROOK => {
                let index = self.rook_magics[square].get_index(occupancy);
                self.rook[index]
            }
            Pieces::BISHOP => {
                let index = self.bishop_magics[square].get_index(occupancy);
                self.bishop[index]
            }
            Pieces::QUEEN => {
                let r_index = self.rook_magics[square].get_index(occupancy);
                let b_index = self.bishop_magics[square].get_index(occupancy);
                self.rook[r_index] ^ self.bishop[b_index]
            }
            _ => panic!("Not a sliding piece: {}", piece),
        }
    }

    /** Return pawn attacks for the given square. */
    pub fn get_pawn_attacks(&self, side: Side, square: Square) -> Bitboard {
        self.pawns[side][square]
    }
}

// *** === Getting the actual pseudo-legal moves. === *** //

impl MoveGenerator {
    pub fn piece(&self, board: &Board, piece: Piece, list: &mut MoveList) {
        let us = board.game_state.active_color as usize;
        let bb_occupancy = board.occupancy();
        let bb_own_pieces = board.bb_pieces[us];
        let mut bb_pieces = board.get_pieces(piece, us);

        // Generate moves for each piece of the type passed into the function.
        while bb_pieces > 0 {
            let from = bits::next(&mut bb_pieces);
            let bb_target = match piece {
                Pieces::KING | Pieces::KNIGHT => board.get_non_slider_attacks(piece, from),
                Pieces::QUEEN | Pieces::ROOK | Pieces::BISHOP => {
                    board.get_slider_attacks(piece, from, bb_occupancy)
                }
                _ => panic!("Not a sliding piece: {}", piece),
            };

            // A piece can move to where there is no piece of our own.
            let bb_moves = bb_target & !bb_own_pieces;
            self.add_move(board, piece, from, bb_moves, list);
        }
    }

    pub fn pawns(&self, board: &Board, list: &mut MoveList) {
        let us = board.game_state.active_color as usize;
        let bb_opponent_pieces = board.bb_pieces[us ^ 1];
        let bb_empty = !board.occupancy();
        let bb_fourth = if us == WHITE {
            BB_RANKS[Ranks::R4]
        } else {
            BB_RANKS[Ranks::R5]
        };
        let mut bb_pawns = board.get_pieces(Pieces::PAWN, us);
        let direction = if us == WHITE { 8 } else { -8 };

        // As long as there are pawns, generate moves for each of them.
        while bb_pawns > 0 {
            let from = bits::next(&mut bb_pawns);
            let bb_push = 1u64 << (from as i8 + direction);
            let bb_one_step = bb_push & bb_empty;
            let bb_two_step =
                bb_one_step.rotate_left((64 + direction) as u32) & bb_empty & bb_fourth;
            let bb_targets = board.get_pawn_attacks(us, from);
            let bb_captures = bb_targets & bb_opponent_pieces;
            let bb_ep_capture = match board.game_state.en_passant {
                Some(ep) => bb_targets & (1u64 << ep),
                None => 0,
            };

            // Gather all moves for the pawn into one bitboard.
            let bb_moves = bb_one_step | bb_two_step | bb_captures | bb_ep_capture;
            self.add_move(board, Pieces::PAWN, from, bb_moves, list);
        }
    }

    pub fn castling(&self, board: &Board, list: &mut MoveList) {
        let us = board.game_state.active_color as usize;
        let opponent = us ^ 1;
        let castle_perms_white = (board.game_state.castling & (Castling::WK | Castling::WQ)) > 0;
        let castle_perms_black = (board.game_state.castling & (Castling::BK | Castling::BQ)) > 0;
        let bb_occupancy = board.occupancy();
        let mut bb_king = board.get_pieces(Pieces::KING, us);
        let from = bits::next(&mut bb_king);

        if us == WHITE && castle_perms_white {
            // Kingside
            if board.game_state.castling & Castling::WK > 0 {
                let bb_kingside_blockers: u64 = (1u64 << Squares::F1) | (1u64 << Squares::G1);
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !info::square_attacked(board, opponent, Squares::E1)
                    && !info::square_attacked(board, opponent, Squares::F1)
                {
                    let to = (1u64 << from) << 2;
                    self.add_move(board, Pieces::KING, from, to, list);
                }
            }

            if board.game_state.castling & Castling::WQ > 0 {
                // Queenside
                let bb_queenside_blockers: u64 =
                    (1u64 << Squares::B1) | (1u64 << Squares::C1) | (1u64 << Squares::D1);
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !info::square_attacked(board, opponent, Squares::E1)
                    && !info::square_attacked(board, opponent, Squares::D1)
                {
                    let to = (1u64 << from) >> 2;
                    self.add_move(board, Pieces::KING, from, to, list);
                }
            }
        }

        if us == BLACK && castle_perms_black {
            // Kingside
            if board.game_state.castling & Castling::BK > 0 {
                let bb_kingside_blockers: u64 = (1u64 << Squares::F8) | (1u64 << Squares::G8);
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !info::square_attacked(board, opponent, Squares::E8)
                    && !info::square_attacked(board, opponent, Squares::F8)
                {
                    let to = (1u64 << from) << 2;
                    self.add_move(board, Pieces::KING, from, to, list);
                }
            }

            // Queenside
            if board.game_state.castling & Castling::BQ > 0 {
                let bb_queenside_blockers: u64 =
                    (1u64 << Squares::B8) | (1u64 << Squares::C8) | (1u64 << Squares::D8);
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !info::square_attacked(board, opponent, Squares::E8)
                    && !info::square_attacked(board, opponent, Squares::D8)
                {
                    let to = (1u64 << from) >> 2;
                    self.add_move(board, Pieces::KING, from, to, list);
                }
            }
        }
    }

    pub fn add_move(
        &self,
        board: &Board,
        piece: Piece,
        from: Square,
        to: Bitboard,
        list: &mut MoveList,
    ) {
        let mut bb_to = to;
        let us = board.game_state.active_color as usize;
        let promotion_rank = if us == WHITE { Ranks::R8 } else { Ranks::R1 };
        let is_pawn = piece == Pieces::PAWN;

        // As long as there are still to-squres in bb_to, this piece has moves to add.
        while bb_to > 0 {
            let to_square = bits::next(&mut bb_to);
            let capture = board.piece_list[to_square];
            let en_passant = match board.game_state.en_passant {
                Some(square) => is_pawn && (square as usize == to_square),
                None => false,
            };
            let promotion = is_pawn && utils::square_on_rank(to_square, promotion_rank);
            let double_step = is_pawn && ((to_square as i8 - from as i8).abs() == 16);
            let castling = (piece == Pieces::KING) && ((to_square as i8 - from as i8).abs() == 2);

            // Gather all data for this move into one 64-bit integer.
            let no_promotion_piece = Pieces::NONE << Shift::PROMOTION;
            let move_data = (piece)
                | from << Shift::FROM_SQ
                | to_square << Shift::TO_SQ
                | capture << Shift::CAPTURE
                | (en_passant as usize) << Shift::EN_PASSANT
                | (double_step as usize) << Shift::DOUBLE_STEP
                | (castling as usize) << Shift::CASTLING
                | no_promotion_piece;

            // If no promomotion, just push the move to the move list. Otherwise,
            // remove the no_promotion_piece from move_data. Then iterate over the
            // promotion pieces, and push each promotion option to the move list.
            match !promotion {
                true => list.push(Move::new(move_data)),
                false => {
                    let reset = move_data ^ no_promotion_piece;
                    PROMOTION_PIECES.iter().for_each(|piece| {
                        let current = *piece << Shift::PROMOTION;
                        let d = reset | current;
                        list.push(Move::new(d))
                    });
                }
            }
        }
    }
}
