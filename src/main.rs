mod board;
mod defines;
mod fen;
mod magics;
mod movegen;
mod print;
mod utils;

use board::Board;
use defines::*;
use magics::Magics;
use movegen::*;

fn main() {
    let mut board: Board = Default::default();
    let mut magics: Magics = Default::default();
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    let test_pos: &str = "8/8/8/8/8/8/8/8 w - - 0 1";
    board.initialize(test_pos);
    magics.initialize();

    for i in 0..=7 {
        print::bitboard(board.bb_ranks[i], None);
    }

    for i in 0..=7 {
        print::bitboard(board.bb_files[i], None);
    }

    // for i in ALL_SQUARES {
    //     print::bitboard(magics.get_non_slider_attacks(KING, i), Some(i));
    // }

    print::engine_info();
    print::position(&board, None);

    movegen::generate(&board, WHITE, &magics, &mut moves);
    print::movelist(&moves);
    movegen::generate(&board, BLACK, &magics, &mut moves);
    print::movelist(&moves);

    println!("Done.");
}
