use crate::{defs::FEN_START_POSITION, misc::cmdline};

pub struct Engine {
    cmdline_fen: String,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            cmdline_fen: String::from(""),
        }
    }

    pub fn cmdline_get_values(&mut self) {
        let cmdline = cmdline::get();
        self.cmdline_fen = cmdline
            .value_of("fen")
            .unwrap_or(FEN_START_POSITION)
            .to_string();
    }

    pub fn run(&mut self) {
        self.cmdline_get_values();
        println!("Running...");
        println!("FEN: {}", self.cmdline_fen);
    }
}
