mod board;
mod defines;
mod fen;
mod magics;
mod movegen;
mod print;

use board::Board;
use defines::*;
use magics::Magics;
use movegen::*;

fn main() {
    let mut board: Board = Default::default();
    let mut magics: Magics = Default::default();
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    let test_pos: &str = "1B6/4kN2/2n1P3/8/1P1pP3/8/1p4p1/R3K2N w - e3 0 1";
    board.initialize(test_pos);
    magics.initialize();

    print::engine_info();
    print::position(&board, None);
    if let Some(s) = board.en_passant {
        println!("en_passant: {}", s);
    };

    movegen::generate(&board, WHITE, &magics, &mut moves);
    print::movelist(&moves);
    movegen::generate(&board, BLACK, &magics, &mut moves);
    print::movelist(&moves);

    println!("Done.");
}
