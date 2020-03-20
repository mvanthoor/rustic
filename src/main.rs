mod board;
mod defs;
mod extra;
mod movegen;
mod utils;

use board::make_move::make_move;
use board::representation::Board;
use board::unmake_move::unmake_move;
use board::zobrist::ZobristRandoms;
use extra::print;
use movegen::movedefs::{MoveList, MAX_LEGAL_MOVES};
use movegen::MoveGenerator;
use utils::engine_info;

fn main() {
    let test_pos: &str = "7k/6RP/8/8/8/8/8/K7 b - - 0 1";
    let move_generator = MoveGenerator::new();
    let zobrist_randoms = ZobristRandoms::new();
    let mut board: Board = Board::new(&zobrist_randoms, Some(test_pos));
    let mut move_list: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    engine_info();

    print::position(&board, None);

    move_generator.gen_all_moves(&board, &mut move_list);
    print::movelist(&move_list);

    for mv in move_list.iter() {
        println!();
        println!("Executing move...");
        print::move_data(&mv);

        make_move(&mut board, *mv);

        println!("After:");
        print::position(&board, None);

        unmake_move(&mut board);

        println!("Undone:");
        print::position(&board, None);
    }

    println!("Finished.");
}
