mod board;
mod defs;
mod evaluation;
mod extra;
// mod interface;
mod misc;
mod movegen;

// use interface::console;
use board::{defs::ERR_FEN_PARTS, Board};
use extra::perft;
use misc::{cmdline, info};
use movegen::MoveGenerator;

fn main() {
    let cmd_line = cmdline::get();
    let x = cmd_line.value_of("experiment").unwrap_or("Not proven.");
    println!("The value of this experiment is: {}", x);

    let test_pos = Some("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let move_generator = MoveGenerator::new();
    let mut board: Board = Board::new();
    let setup_result = board.fen_read(test_pos);

    info::about();

    match setup_result {
        Ok(()) => perft::run(&board, 6, &move_generator), //while console::get_input(&mut board) != 0 {},
        Err(e) => println!("Error in FEN-part: {}", ERR_FEN_PARTS[e as usize]),
    }
}
