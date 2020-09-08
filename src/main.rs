mod board;
mod defs;
mod engine;
mod evaluation;
mod extra;
mod interface;
mod misc;
mod movegen;

// use interface::console;
use engine::Engine;
use misc::info;

fn main() {
    let mut engine = Engine::new();

    info::about();
    engine.run();

    // perft::run(&board, 6, &move_generator), //while
    // console::get_input(&mut board) != 0 {},
}
