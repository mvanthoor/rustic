mod create;
pub mod defs;
mod init;
mod magics;
mod movelist;

use crate::{
    board::defs::{Pieces, Squares, BB_RANKS, BB_SQUARES},
    board::Board,
    defs::{Bitboard, Castling, NrOf, Piece, Side, Sides, Square, EMPTY, MAX_LEGAL_MOVES},
    misc::bits,
};
use defs::{Move, MoveType, Shift};
use magics::Magic;
use movelist::MoveList;

use self::defs::MoveListRaw;

// This is a list of all pieces a pawn can promote to.
const PROMOTION_PIECES: [usize; 4] = [Pieces::QUEEN, Pieces::ROOK, Pieces::BISHOP, Pieces::KNIGHT];

// These are the exact sizes needed for the rook and bishop moves. These
// can be calculated by adding all the possible blocker boards for a rook
// or a bishop.
pub const ROOK_TABLE_SIZE: usize = 102_400; // Total permutations of all rook blocker boards.
pub const BISHOP_TABLE_SIZE: usize = 5_248; // Total permutations of all bishop blocker boards.

// The move generator struct holds the attack table for each piece, and the
// tables with magic numbers for the rook and bishop.
pub struct MoveGenerator {
    king: [Bitboard; NrOf::SQUARES],
    knight: [Bitboard; NrOf::SQUARES],
    pawns: [[Bitboard; NrOf::SQUARES]; Sides::BOTH],
    rook: Vec<Bitboard>,
    bishop: Vec<Bitboard>,
    rook_magics: [Magic; NrOf::SQUARES],
    bishop_magics: [Magic; NrOf::SQUARES],
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveGenerator {
    // Creates a new move generator and initializes all the tables.
    pub fn new() -> Self {
        let magics: Magic = Default::default();
        let mut mg = Self {
            king: [EMPTY; NrOf::SQUARES],
            knight: [EMPTY; NrOf::SQUARES],
            pawns: [[EMPTY; NrOf::SQUARES]; Sides::BOTH],
            rook: vec![EMPTY; ROOK_TABLE_SIZE],
            bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            rook_magics: [magics; NrOf::SQUARES],
            bishop_magics: [magics; NrOf::SQUARES],
        };
        mg.init_king();
        mg.init_knight();
        mg.init_pawns();
        mg.init_magics(Pieces::ROOK);
        mg.init_magics(Pieces::BISHOP);
        mg
    }

    // Generates moves for the side that is to move.
    pub fn generate_moves(
        &self,
        board: &Board,
        memory: &mut MoveListRaw,
        move_type: MoveType,
    ) -> MoveList {
        let mut count: u8 = 0;

        self.piece(board, Pieces::KING, memory, move_type, &mut count);
        self.piece(board, Pieces::KNIGHT, memory, move_type, &mut count);
        self.piece(board, Pieces::ROOK, memory, move_type, &mut count);
        self.piece(board, Pieces::BISHOP, memory, move_type, &mut count);
        self.piece(board, Pieces::QUEEN, memory, move_type, &mut count);
        self.pawns(board, memory, move_type, &mut count);

        if move_type == MoveType::All || move_type == MoveType::Quiet {
            self.castling(board, memory, &mut count);
        }

        MoveList::new(memory, count)
    }

    // Return non-slider (King, Knight) attacks for the given square.
    pub fn get_non_slider_attacks(&self, piece: Piece, square: Square) -> Bitboard {
        match piece {
            Pieces::KING => self.king[square],
            Pieces::KNIGHT => self.knight[square],
            _ => panic!("Not a king or a knight: {piece}"),
        }
    }

    // Return slider attack for Rook, Bishop and Queen using the magic numbers.
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
            _ => panic!("Not a sliding piece: {piece}"),
        }
    }

    // Return pawn attacks for the given square.
    pub fn get_pawn_attacks(&self, side: Side, square: Square) -> Bitboard {
        self.pawns[side][square]
    }
}

// *** === Getting the actual pseudo-legal moves. === *** //

impl MoveGenerator {
    pub fn piece(
        &self,
        board: &Board,
        piece: Piece,
        memory: &mut MoveListRaw,
        move_type: MoveType,
        count: &mut u8,
    ) {
        let us = board.us();
        let bb_occupancy = board.occupancy();

        // Get squares that are empty, occupied by our own pieces, and occupied by
        // our opponent's pieces.
        let bb_empty = !bb_occupancy;
        let bb_own_pieces = board.bb_side[us];
        let bb_opponent_pieces = board.bb_side[board.opponent()];

        let mut bb_pieces = board.get_pieces(us, piece);

        // Generate moves for each piece of the type passed into the function.
        while bb_pieces > 0 {
            let from = bits::next(&mut bb_pieces);
            let bb_target = match piece {
                Pieces::KING | Pieces::KNIGHT => self.get_non_slider_attacks(piece, from),
                Pieces::QUEEN | Pieces::ROOK | Pieces::BISHOP => {
                    self.get_slider_attacks(piece, from, bb_occupancy)
                }
                _ => panic!("Not a piece: {piece}"),
            };

            // Generate moves according to requested move type.
            let bb_moves = match move_type {
                MoveType::All => bb_target & !bb_own_pieces,
                MoveType::Quiet => bb_target & bb_empty,
                MoveType::Capture => bb_target & bb_opponent_pieces,
            };

            self.add_move(board, piece, from, bb_moves, memory, count);
        }
    }

    pub fn pawns(
        &self,
        board: &Board,
        memory: &mut MoveListRaw,
        move_type: MoveType,
        count: &mut u8,
    ) {
        // Create shorthand variables.
        let us = board.us();
        let bb_opponent_pieces = board.bb_side[board.opponent()];
        let bb_empty = !board.occupancy();
        let bb_fourth = BB_RANKS[Board::fourth_rank(us)];
        let direction = Board::pawn_direction(us);
        let rotation_count = (NrOf::SQUARES as i8 + direction) as u32;
        let mut bb_pawns = board.get_pieces(us, Pieces::PAWN);

        // As long as there are pawns, generate moves for each of them.
        while bb_pawns > 0 {
            let from = bits::next(&mut bb_pawns);
            let to = (from as i8 + direction) as usize;
            let mut bb_moves = 0;

            // Generate pawn pushes
            if move_type == MoveType::All || move_type == MoveType::Quiet {
                let bb_push = BB_SQUARES[to];
                let bb_one_step = bb_push & bb_empty;
                let bb_two_step = bb_one_step.rotate_left(rotation_count) & bb_empty & bb_fourth;
                bb_moves |= bb_one_step | bb_two_step;
            }

            // Generate pawn captures
            if move_type == MoveType::All || move_type == MoveType::Capture {
                let bb_targets = self.get_pawn_attacks(us, from);
                let bb_captures = bb_targets & bb_opponent_pieces;
                let bb_ep_capture = match board.game_state.en_passant {
                    Some(ep) => bb_targets & BB_SQUARES[ep as usize],
                    None => 0,
                };
                bb_moves |= bb_captures | bb_ep_capture;
            }

            self.add_move(board, Pieces::PAWN, from, bb_moves, memory, count);
        }
    }

    pub fn castling(&self, board: &Board, memory: &mut MoveListRaw, count: &mut u8) {
        // Create shorthand variables.
        let us = board.us();
        let opponent = board.opponent();
        let castle_perms_white = (board.game_state.castling & (Castling::WK | Castling::WQ)) > 0;
        let castle_perms_black = (board.game_state.castling & (Castling::BK | Castling::BQ)) > 0;
        let bb_occupancy = board.occupancy();
        let mut bb_king = board.get_pieces(us, Pieces::KING);
        let from = bits::next(&mut bb_king);

        // Generate castling moves for white.
        if us == Sides::WHITE && castle_perms_white {
            // Kingside
            if board.game_state.castling & Castling::WK > 0 {
                let bb_kingside_blockers = BB_SQUARES[Squares::F1] | BB_SQUARES[Squares::G1];
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !self.square_attacked(board, opponent, Squares::E1)
                    && !self.square_attacked(board, opponent, Squares::F1)
                {
                    let to = BB_SQUARES[from] << 2;
                    self.add_move(board, Pieces::KING, from, to, memory, count);
                }
            }

            if board.game_state.castling & Castling::WQ > 0 {
                // Queenside
                let bb_queenside_blockers =
                    BB_SQUARES[Squares::B1] | BB_SQUARES[Squares::C1] | BB_SQUARES[Squares::D1];
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !self.square_attacked(board, opponent, Squares::E1)
                    && !self.square_attacked(board, opponent, Squares::D1)
                {
                    let to = BB_SQUARES[from] >> 2;
                    self.add_move(board, Pieces::KING, from, to, memory, count);
                }
            }
        }

        // Generate castling moves for black.
        if us == Sides::BLACK && castle_perms_black {
            // Kingside
            if board.game_state.castling & Castling::BK > 0 {
                let bb_kingside_blockers = BB_SQUARES[Squares::F8] | BB_SQUARES[Squares::G8];
                let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

                if !is_kingside_blocked
                    && !self.square_attacked(board, opponent, Squares::E8)
                    && !self.square_attacked(board, opponent, Squares::F8)
                {
                    let to = BB_SQUARES[from] << 2;
                    self.add_move(board, Pieces::KING, from, to, memory, count);
                }
            }

            // Queenside
            if board.game_state.castling & Castling::BQ > 0 {
                let bb_queenside_blockers =
                    BB_SQUARES[Squares::B8] | BB_SQUARES[Squares::C8] | BB_SQUARES[Squares::D8];
                let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

                if !is_queenside_blocked
                    && !self.square_attacked(board, opponent, Squares::E8)
                    && !self.square_attacked(board, opponent, Squares::D8)
                {
                    let to = BB_SQUARES[from] >> 2;
                    self.add_move(board, Pieces::KING, from, to, memory, count);
                }
            }
        }
    }

    // Add the generated moves to the move list.
    pub fn add_move(
        &self,
        board: &Board,
        piece: Piece,
        from: Square,
        to: Bitboard,
        memory: &mut MoveListRaw,
        count: &mut u8,
    ) {
        // Shorthand variables.
        let mut bb_to = to;
        let us = board.us();
        let promotion_rank = Board::promotion_rank(us);
        let is_pawn = piece == Pieces::PAWN;

        // As long as there are still to-squares in bb_to, this piece has moves to add.
        while bb_to > 0 {
            // More shorthand variables
            let to_square = bits::next(&mut bb_to);
            let capture = board.piece_list[to_square];
            let en_passant = match board.game_state.en_passant {
                Some(square) => is_pawn && (square as usize == to_square),
                None => false,
            };
            let promotion = is_pawn && Board::square_on_rank(to_square, promotion_rank);
            let double_step = is_pawn && ((to_square as i8 - from as i8).abs() == 16);
            let castling = (piece == Pieces::KING) && ((to_square as i8 - from as i8).abs() == 2);

            // Gather all data for this move into one 64-bit integer.
            let mut move_data = (piece)
                | from << Shift::FROM_SQ
                | to_square << Shift::TO_SQ
                | capture << Shift::CAPTURE
                | (en_passant as usize) << Shift::EN_PASSANT
                | (double_step as usize) << Shift::DOUBLE_STEP
                | (castling as usize) << Shift::CASTLING;

            // Push the move to the piece list...
            if !promotion {
                move_data |= Pieces::NONE << Shift::PROMOTION;
                memory[*count as usize].write(Move::new(move_data));
                *count += 1;
            } else {
                // ...or push four promotion moves.
                PROMOTION_PIECES.iter().for_each(|piece| {
                    let promotion_piece = *piece << Shift::PROMOTION;
                    memory[*count as usize].write(Move::new(move_data | promotion_piece));
                    *count += 1;
                });
            }
        }
    }
}

impl MoveGenerator {
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    // Determine if a square is attacked by 'attacker', on the given board.
    pub fn square_attacked(&self, board: &Board, attacker: Side, square: Square) -> bool {
        let attackers = board.bb_pieces[attacker];

        // Use the super-piece method: get the moves for each piece,
        // starting from the given square. This provides the squares where
        // a piece has to be, to be able to reach the given square.
        let occupancy = board.occupancy();
        let bb_king = self.get_non_slider_attacks(Pieces::KING, square);
        let bb_rook = self.get_slider_attacks(Pieces::ROOK, square, occupancy);
        let bb_bishop = self.get_slider_attacks(Pieces::BISHOP, square, occupancy);
        let bb_knight = self.get_non_slider_attacks(Pieces::KNIGHT, square);
        let bb_pawns = self.get_pawn_attacks(attacker ^ 1, square);
        let bb_queen = bb_rook | bb_bishop;

        // Then determine if such a piece is actually there: see if a rook
        // is on one of the squares a rook has to be to reach the given
        // square. Same for the queen, knight, etc... As soon as one is
        // found, the square is attacked.
        (bb_king & attackers[Pieces::KING] > 0)
            || (bb_rook & attackers[Pieces::ROOK] > 0)
            || (bb_queen & attackers[Pieces::QUEEN] > 0)
            || (bb_bishop & attackers[Pieces::BISHOP] > 0)
            || (bb_knight & attackers[Pieces::KNIGHT] > 0)
            || (bb_pawns & attackers[Pieces::PAWN] > 0)
    }
}
