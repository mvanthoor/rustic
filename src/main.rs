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
use movegen::information::square_attacked;
use movegen::movedefs::{MoveList, MAX_LEGAL_MOVES};
use movegen::MoveGenerator;
use utils::engine_info;

fn main() {
    let test_pos: &str = "7k/8/8/8/3pP3/8/8/7K b - e3 0 1";
    let move_generator = MoveGenerator::new();
    let zobrist_randoms = ZobristRandoms::new();
    let mut board: Board = Board::new(&zobrist_randoms, Some(test_pos));
    let mut move_list: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    engine_info();

    print::position(&board, None);

    move_generator.gen_all_moves(&board, &mut move_list);
    print::movelist(&move_list);

    let mut has_move = false;
    for mv in move_list.iter() {
        println!();
        println!("Executing move...");
        print::move_data(&mv);

        let is_legal = make_move(&mut board, *mv, &move_generator);

        if !is_legal {
            println!("This move is not legal. King in check after move.");
        } else {
            has_move = true;
            println!("After:");
            print::position(&board, None);
            unmake_move(&mut board);
            println!("Undone:");
            print::position(&board, None);
        }
    }

    if !has_move {
        let king_square = board.bb_w[0].trailing_zeros() as u8;
        let in_check = square_attacked(&board, 1, &move_generator, king_square);
        if in_check {
            println!("Checkmate.");
        } else {
            println!("Stalemate.");
        }
    }

    println!("Finished.");
}
