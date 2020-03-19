use super::representation::{Board, UnMake};
use crate::movegen::movedefs::Move;

pub fn make_move(board: &mut Board, m: Move) {
    let unmake_info = UnMake::new(
        board.active_color,
        board.castling,
        board.en_passant,
        board.halfmove_clock,
        board.fullmove_number,
        m,
    );
    board.unmake_list.push(unmake_info);
}
