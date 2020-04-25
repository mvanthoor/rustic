mod board;
mod defs;
mod evaluation;
mod extra;
mod interface;
mod movegen;
mod search;
mod utils;

use board::{fen::ERR_FEN_PARTS, representation::Board, zobrist::ZobristRandoms};
use interface::console;
use movegen::MoveGenerator;
use std::sync::Arc;
use utils::parse;

fn main() {
    let test_pos = Some("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let move_generator = MoveGenerator::new();
    let zobrist_randoms = ZobristRandoms::new();
    let mut board: Board = Board::new(Arc::new(zobrist_randoms), Arc::new(move_generator));
    let setup_result = board.fen_read(None);

    utils::engine_info();

    match setup_result {
        Ok(()) => while console::get_input(&mut board) != 0 {},
        Err(e) => println!("Error in FEN-part: {}", ERR_FEN_PARTS[e as usize]),
    }
}
