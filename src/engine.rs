use crate::{
    board::{defs::Pieces, Board},
    comm,
    defs::{About, FEN_START_POSITION},
    extra::{perft, wizardry},
    misc::cmdline,
    movegen::MoveGenerator,
};
use clap::ArgMatches;

struct CmdLine {
    arguments: ArgMatches<'static>,
}

impl CmdLine {
    fn comm(&self) -> String {
        self.arguments.value_of("comm").unwrap_or("uci").to_string()
    }

    fn fen(&self) -> String {
        self.arguments
            .value_of("fen")
            .unwrap_or(FEN_START_POSITION)
            .to_string()
    }

    fn perft(&self) -> u8 {
        self.arguments
            .value_of("perft")
            .unwrap_or("1")
            .parse()
            .unwrap_or(1)
    }

    fn wizardry(&self) -> bool {
        self.arguments.is_present("wizardry")
    }
}

pub struct Engine {
    cmdline: CmdLine,
    move_generator: MoveGenerator,
    board: Board,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            cmdline: CmdLine {
                arguments: cmdline::get(),
            },
            move_generator: MoveGenerator::new(),
            board: Board::new(),
        }
    }

    pub fn run(&mut self) {
        let fen = &self.cmdline.fen()[..];
        self.about();
        comm::uci::run();

        println!("Engine running...");
        println!("FEN: {}", self.cmdline.fen());

        self.board.fen_read(Some(fen));
        perft::run(&self.board, 6, &self.move_generator);

        if self.cmdline.wizardry() {
            wizardry::find_magics(Pieces::ROOK);
            wizardry::find_magics(Pieces::BISHOP);
        }
    }

    fn about(&self) {
        println!();
        println!("Engine: {} {}", About::ENGINE, About::VERSION);
        println!("Author: {} <{}>", About::AUTHOR, About::EMAIL);
        println!("Description: {}", About::DESCRIPTION);
    }
}
