mod board;
mod defs;
mod extra;
mod movegen;
mod perft;
mod utils;

use board::representation::Board;
use board::zobrist::ZobristRandoms;
use extra::print;
use movegen::movedefs::MoveList;
use movegen::MoveGenerator;
use utils::engine_info;

fn main() {
    let test_pos: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let move_generator = MoveGenerator::new();
    let zobrist_randoms = ZobristRandoms::new();
    let mut board: Board = Board::new(&zobrist_randoms, None);
    //let mut move_list: MoveList = MoveList::new();

    engine_info();
    print::position(&board, None);

    perft::run(&board, 7, &move_generator);

    println!("Finished.");
}
