mod board;
mod defs;
mod evaluation;
mod extra;
// mod interface;
mod engine;
mod misc;
mod movegen;

use board::defs::ERR_FEN_PARTS;
// use interface::console;
use engine::Engine;

fn main() {
    let test_pos = Some("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let mut engine = Engine::new();
    let fen_setup_result = engine.position_setup(test_pos);

    engine.about();

    match fen_setup_result {
        Ok(()) => engine.perft(6), //while console::get_input(&mut board) != 0 {},
        Err(e) => println!("Error in FEN-part: {}", ERR_FEN_PARTS[e as usize]),
    }
}
