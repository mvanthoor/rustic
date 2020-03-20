use super::representation::{Board, UnMakeInfo};
use crate::defs::WHITE;
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

    let side = board.active_color as usize;
    let piece = m.piece() as usize;
    let from = m.from();
    let to = m.to();
    let bb = if side == WHITE {
        &mut board.bb_w[..]
    } else {
        &mut board.bb_b[..]
    };

    // take the piece out of the key, on the from-square.
    board.zobrist_key ^= board.zobrist_randoms.piece(side, piece, from);

    // actually move the piece in the bitboards
    clear_bit(&mut bb[piece], from);
    clear_bit(&mut board.bb_pieces[side], from);
    set_bit(&mut bb[piece], to);
    set_bit(&mut board.bb_pieces[side], to);

    // add the piece into the key, on the to-square.
    board.zobrist_key ^= board.zobrist_randoms.piece(side, piece, to);

    board.active_color = (side ^ 1) as u8;
    board.fullmove_number += 1;
    board.unmake_list.push(unmake_info);
}
