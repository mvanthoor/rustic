mod board;
mod comm;
mod defs;
mod engine;
mod evaluation;
mod extra;
mod misc;
mod movegen;

// use interface::console;
use defs::ENGINE_RUN_ERRORS;
use engine::Engine;

fn main() {
    let mut engine = Engine::new();
    let result = engine.run();

    match result {
        Ok(()) => (),
        Err(e) => println!("Error code {}: {}", e, ENGINE_RUN_ERRORS[e as usize]),
    };
}
