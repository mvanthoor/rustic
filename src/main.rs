mod board;
mod comm;
mod defs;
mod engine;
mod evaluation;
mod extra;
mod misc;
mod movegen;

// use interface::console;
use engine::Engine;

fn main() {
    let mut engine = Engine::new();
    engine.run();
}
