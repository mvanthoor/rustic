use super::representation::{Board, UnMakeInfo};
use crate::defs::{Bitboard, PNONE, WHITE};
use crate::movegen::movedefs::Move;
use crate::print;
use crate::utils::{clear_bit, set_bit};

pub fn make_move(board: &mut Board, m: Move) {
    let unmake_info = UnMakeInfo::new(
        board.active_color,
        board.castling,
        board.en_passant,
        board.halfmove_clock,
        board.fullmove_number,
        board.zobrist_key,
        m,
    );

    // Set "us" and "opponent"
    let us = board.active_color as usize;
    let opponent = (us ^ 1) as usize;

    // Set which bitboards are "us" and "opponent" pieces
    let bb_mine: &mut [Bitboard];
    let bb_opponent: &mut [Bitboard];

    if us == WHITE {
        bb_mine = &mut board.bb_w[..];
        bb_opponent = &mut board.bb_b[..];
    } else {
        bb_mine = &mut board.bb_b[..];
        bb_opponent = &mut board.bb_w[..];
    };

    // Dissect the move
    let piece = m.piece() as usize;
    let from = m.from();
    let to = m.to();
    let captured = m.captured() as usize;

    // If a piece is captured by this move, then remove it from the to-square
    if captured != PNONE {
        board.zobrist_key ^= board.zobrist_randoms.piece(opponent, captured, to);
        clear_bit(&mut bb_opponent[captured], to);
        clear_bit(&mut board.bb_pieces[opponent], to);
    }

    // take the moving piece off the from-square
    board.zobrist_key ^= board.zobrist_randoms.piece(us, piece, from);
    clear_bit(&mut bb_mine[piece], from);
    clear_bit(&mut board.bb_pieces[us], from);

    // put the moving piece on the to-square
    set_bit(&mut bb_mine[piece], to);
    set_bit(&mut board.bb_pieces[us], to);
    board.zobrist_key ^= board.zobrist_randoms.piece(us, piece, to);

    board.active_color = opponent as u8;
    board.fullmove_number += 1;
    board.unmake_list.push(unmake_info);
}
