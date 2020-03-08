mod board;
mod defines;
mod fen;
mod movegenerator;
mod print;
mod utils;

use board::Board;
use defines::*;
use movegenerator::gen::{square_attacked, MoveList, MAX_LEGAL_MOVES};
use movegenerator::init::Movements;
use utils::*;

fn main() {
    let mut board: Board = Default::default();
    let mut movements: Movements = Default::default();
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    let test_pos: &str = "4k3/6P1/2P1P3/8/8/8/6p1/3K4 w - - 0 1";
    board.initialize(test_pos);
    movements.initialize();

    // engine_info();

    print::position(&board, None);

    let x = square_attacked(&board, WHITE, &movements, 62);
    println!("{}", x);

    // movegenerator::gen::generate(&board, WHITE, &movements, &mut moves);
    // print::movelist(&moves);
    // println!("...");
    // movegenerator::gen::generate(&board, BLACK, &movements, &mut moves);
    // print::movelist(&moves);

    println!("Done.");
}
