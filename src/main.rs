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
use utils::*;

fn main() {
    let mut board: Board = Default::default();
    let mut magics: Magics = Default::default();
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    let test_pos: &str = "8/ppp5/P2N4/1P6/2P2pP1/8/4p2p/6R1 w - g3 0 1";
    board.initialize(test_pos);
    magics.initialize();

    engine_info();

    print::position(&board, None);

    // movegen::generate(&board, WHITE, &magics, &mut moves);
    // print::movelist(&moves);
    // println!("...");
    // movegen::generate(&board, BLACK, &magics, &mut moves);
    // print::movelist(&moves);

    // let blocker = (1u64 << 26) | (1u64 << 52) | (1u64 << 29) | (1u64 << 42) | (1u64 << 11);

    // for i in ALL_SQUARES {
    //     println!("square: {}", i);
    //     print::bitboard(blocker, Some(i));

    //     let a = create_bb_ray(blocker, i, Direction::Up);
    //     let b = create_bb_ray(blocker, i, Direction::Right);
    //     let c = create_bb_ray(blocker, i, Direction::Down);
    //     let d = create_bb_ray(blocker, i, Direction::Left);
    //     let bb = a | b | c | d;

    //     print::bitboard(bb, Some(i));
    // }

    println!("Done.");
}
