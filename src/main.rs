mod board;
mod defs;
mod engine;
mod evaluation;
mod extra;
mod interface;
mod misc;
mod movegen;

// use interface::console;
use board::Board;
use engine::Engine;
use misc::info;

fn main() {
    let test_pos = Some("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let mut board: Board = Board::new();
    let setup_result = board.fen_read(test_pos);

    let mut engine = Engine::new();

    engine.run();

    info::about();

    // perft::run(&board, 6, &move_generator), //while
    // console::get_input(&mut board) != 0 {},

    match setup_result {
        Ok(()) => interface::uci::run(),
        Err(_) => println!("Error..."),
    }
}
