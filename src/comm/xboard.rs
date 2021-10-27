/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

// This file implements the XBoard communication module.

use super::{shared::Shared, CommIn, CommInfo, CommOut, CommType, IComm};
use crate::{
    board::Board,
    defs::About,
    engine::defs::{EngineOption, EngineState, ErrFatal, Information},
    movegen::defs::Move,
    search::defs::{SearchCurrentMove, SearchStats, SearchSummary},
};
use crossbeam_channel::{self, Sender};
use std::{
    fmt::{self, Display},
    io::{self},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

const FEATURES: [&str; 11] = [
    "done=0",
    "myname=x",
    "ping=1",
    "memory=1",
    "setboard=1",
    "usermove=1",
    "debug=1",
    "draw=0",
    "sigint=0",
    "sigterm=0",
    "done=1",
];

struct Stat01 {
    depth: i8,
    time: u128,
    nodes: usize,
    curr_move: Move,
    curr_move_number: u8,
    legal_moves_total: u8,
}

struct TimeControl {
    sd: u8,
    st: u128,
}

impl TimeControl {
    fn new() -> Self {
        Self { sd: 0, st: 0 }
    }
}

// This struct is used to instantiate the Comm Console module.
pub struct XBoard {
    receiving_handle: Option<JoinHandle<()>>, // Thread for receiving input.
    output_handle: Option<JoinHandle<()>>,    // Thread for sending output.
    output_tx: Option<Sender<CommOut>>,       // Actual output sender object.
    info: CommInfo,
}

#[derive(PartialEq, Clone)]
pub enum XBoardIn {
    XBoard,
    ProtoVer(u8),
    New,
    Force,
    SetBoard(String),
    UserMove(String),
    Ping(i8),
    Post,
    NoPost,
    Memory(usize),
    Analyze,
    Dot,
    Exit,
}

impl Display for XBoardIn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XBoardIn::XBoard => write!(f, "xboard"),
            XBoardIn::ProtoVer(x) => write!(f, "protover {}", x),
            XBoardIn::New => write!(f, "new"),
            XBoardIn::Force => write!(f, "force"),
            XBoardIn::SetBoard(x) => write!(f, "setboard {}", x),
            XBoardIn::UserMove(x) => write!(f, "usermove {}", x),
            XBoardIn::Ping(x) => write!(f, "ping {}", x),
            XBoardIn::Post => write!(f, "post"),
            XBoardIn::NoPost => write!(f, "nopost"),
            XBoardIn::Memory(x) => write!(f, "memory {}", x),
            XBoardIn::Analyze => write!(f, "analyze"),
            XBoardIn::Dot => write!(f, "."),
            XBoardIn::Exit => write!(f, "exit"),
        }
    }
}

pub enum XBoardOut {
    NewLine,
    Features,
    Stat01,
    IllegalMove(String),
    Pong(i8),
}

// Public functions
impl XBoard {
    // Create a new console.
    pub fn new() -> Self {
        Self {
            receiving_handle: None,
            output_handle: None,
            output_tx: None,
            info: CommInfo::new(CommType::XBOARD, false, EngineState::Observing),
        }
    }
}

// Any communication module must implement the trait IComm.
impl IComm for XBoard {
    fn init(
        &mut self,
        receiving_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<EngineOption>>,
    ) {
        // Start threads
        self.input_thread(receiving_tx);
        self.output_thread(board, options);
    }

    // The engine thread (which is the creator of the Comm module) can use
    // this function to send out of the engine onto the console, or towards
    // a user interface.
    fn send(&self, msg: CommOut) {
        if let Some(tx) = &self.output_tx {
            tx.send(msg).expect(ErrFatal::CHANNEL);
        }
    }

    // After the engine sends 'quit' to the control thread, it will call
    // wait_for_shutdown() and then wait here until shutdown is completed.
    fn wait_for_shutdown(&mut self) {
        if let Some(h) = self.receiving_handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }

        if let Some(h) = self.output_handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }

    fn info(&self) -> &CommInfo {
        &self.info
    }
}

// Implement the receiving thread
impl XBoard {
    // The receiving thread receives incoming commands from the console or
    // GUI, which is turns into a "CommIn" object. It sends this
    // object to the engine thread so the engine can decide what to do.
    fn input_thread(&mut self, receiving_tx: Sender<Information>) {
        // Create thread-local variables
        let mut t_incoming_data = String::from(""); // Buffer for incoming data.
        let t_receiving_tx = receiving_tx; // Sends incoming data to engine thread.

        // Actual thread creation.
        let receiving_handle = thread::spawn(move || {
            let mut quit = false;

            // Keep running as long as 'quit' is not detected.
            while !quit {
                // Get data from stdin.
                io::stdin()
                    .read_line(&mut t_incoming_data)
                    .expect(ErrFatal::READ_IO);

                // Create the CommIn object.
                let comm_received = XBoard::create_comm_input(&t_incoming_data);

                // Send it to the engine thread.
                t_receiving_tx
                    .send(Information::Comm(comm_received.clone()))
                    .expect(ErrFatal::HANDLE);

                // Terminate the receiving thread if "Quit" was detected.
                quit = comm_received == CommIn::Quit;

                // Clear for next input
                t_incoming_data = String::from("");
            }
        });

        // Store the handle.
        self.receiving_handle = Some(receiving_handle);
    }
}

// Implement receiving/parsing functions
impl XBoard {
    // This function turns the incoming data into CommIns which the
    // engine is able to understand and react to.
    fn create_comm_input(input: &str) -> CommIn {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match i {
            cmd if cmd == "xboard" => CommIn::XBoard(XBoardIn::XBoard),
            cmd if cmd == "new" => CommIn::XBoard(XBoardIn::New),
            cmd if cmd == "force" => CommIn::XBoard(XBoardIn::Force),
            cmd if cmd == "post" => CommIn::XBoard(XBoardIn::Post),
            cmd if cmd == "nopost" => CommIn::XBoard(XBoardIn::NoPost),
            cmd if cmd == "analyze" => CommIn::XBoard(XBoardIn::Analyze),
            cmd if cmd == "." => CommIn::XBoard(XBoardIn::Dot),
            cmd if cmd == "exit" => CommIn::XBoard(XBoardIn::Exit),
            cmd if cmd == "quit" || cmd.is_empty() => CommIn::Quit,
            cmd if cmd.starts_with("ping") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("protover") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("setboard") => XBoard::parse_setboard(&cmd),
            cmd if cmd.starts_with("usermove") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("memory") => XBoard::parse_key_value_pair(&cmd),

            // Custom commands
            cmd if cmd == "board" => CommIn::Board,
            cmd if cmd == "history" => CommIn::History,
            cmd if cmd == "eval" => CommIn::Eval,
            cmd if cmd == "state" => CommIn::State,
            cmd if cmd == "help" => CommIn::Help,

            // Ignore these commands. May GUI's send them, but the  engine
            // doesn't have any use for them (yet).
            cmd if cmd == "easy" || cmd == "hard" => CommIn::Ignore(cmd),
            cmd if cmd == "random" || cmd.starts_with("accepted") => CommIn::Ignore(cmd),

            // Try to parse anything else as a move.
            _ => XBoard::parse_move(&i),
        }
    }

    fn parse_key_value_pair(cmd: &str) -> CommIn {
        const KEY: usize = 0;
        const VALUE: usize = 1;
        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();

        // Key-value pair has to have two parts. Ignore anything else after
        // the second part.
        if parts.len() >= 2 {
            match &parts[KEY][..] {
                "ping" => {
                    let value = parts[VALUE].parse::<i8>().unwrap_or(0);
                    CommIn::XBoard(XBoardIn::Ping(value))
                }
                "protover" => {
                    let value = parts[VALUE].parse::<u8>().unwrap_or(0);
                    CommIn::XBoard(XBoardIn::ProtoVer(value))
                }
                "memory" => {
                    let value = parts[VALUE].parse::<usize>().unwrap_or(0);
                    CommIn::XBoard(XBoardIn::Memory(value))
                }
                "usermove" => {
                    let value = parts[VALUE].to_lowercase();
                    CommIn::XBoard(XBoardIn::UserMove(value))
                }

                _ => CommIn::Unknown(cmd.to_string()),
            }
        } else {
            CommIn::Unknown(cmd.to_string())
        }
    }

    fn parse_setboard(cmd: &str) -> CommIn {
        let fen = cmd.replace("setboard", "").trim().to_string();
        CommIn::XBoard(XBoardIn::SetBoard(fen))
    }

    fn parse_move(cmd: &str) -> CommIn {
        const ALPHA_COORDS: &str = "abcdefgh";
        const DIGIT_COORDS: &str = "12345678";
        const PROMOTIONS: &str = "qrbn";

        // Count the number of correct characters.
        let mut char_ok = 0;

        // A move is always 4 or 5 characters long.
        if cmd.len() == 4 || cmd.len() == 5 {
            // Check correctness of each character.
            for (i, char) in cmd.chars().enumerate() {
                match i {
                    0 | 2 if ALPHA_COORDS.contains(char) => char_ok += 1,
                    1 | 3 if DIGIT_COORDS.contains(char) => char_ok += 1,
                    4 if PROMOTIONS.contains(char) => char_ok += 1,
                    _ => (),
                }
            }
        }

        // If all characters are OK, then this is a plausible move.
        // Otherwise, it is an unknown command.
        if cmd.len() == char_ok {
            CommIn::XBoard(XBoardIn::UserMove(cmd.to_string()))
        } else {
            CommIn::Unknown(cmd.to_string())
        }
    }
}

// Implement the output thread
impl XBoard {
    // The control thread receives commands from the engine thread.
    fn output_thread(&mut self, board: Arc<Mutex<Board>>, options: Arc<Vec<EngineOption>>) {
        // Create an incoming channel for the control thread.
        let (output_tx, output_rx) = crossbeam_channel::unbounded::<CommOut>();

        // Create the output thread.
        let output_handle = thread::spawn(move || {
            let mut quit = false;
            let _t_options = Arc::clone(&options);

            // In the XBoard-protocol, the engine does not output stats all
            // the time like it does in UCI. It sends the stats when the
            // "." command comes in from the GUI. Therefore the output
            // thread will cache the received stats in this struct, ready
            // to send when the GUI asks for them.
            let mut stat_buf = Stat01 {
                depth: 0,
                time: 0,
                nodes: 0,
                curr_move: Move::new(0),
                curr_move_number: 0,
                legal_moves_total: 0,
            };

            // Keep running as long as Quit is not received.
            while !quit {
                let output = output_rx.recv().expect(ErrFatal::CHANNEL);

                // Perform command as sent by the engine thread.
                match output {
                    CommOut::XBoard(XBoardOut::NewLine) => XBoard::new_line(),
                    CommOut::XBoard(XBoardOut::Features) => XBoard::features(),
                    CommOut::XBoard(XBoardOut::Stat01) => XBoard::stat01(&stat_buf),
                    CommOut::XBoard(XBoardOut::Pong(v)) => XBoard::pong(v),
                    CommOut::XBoard(XBoardOut::IllegalMove(m)) => XBoard::illegal_move(&m),
                    CommOut::BestMove(m) => XBoard::best_move(&m),
                    CommOut::SearchSummary(summary) => {
                        XBoard::search_summary(&mut stat_buf, &summary)
                    }
                    CommOut::SearchStats(stats) => XBoard::search_stats(&mut stat_buf, &stats),
                    CommOut::SearchCurrMove(scm) => {
                        XBoard::search_current_move(&mut stat_buf, &scm)
                    }
                    CommOut::Message(msg) => XBoard::message(&msg),
                    CommOut::Error(err_type, cmd) => XBoard::error(&err_type, &cmd),
                    CommOut::Quit => quit = true,

                    // Custom prints for use in the console.
                    CommOut::PrintBoard => Shared::print_board(&board),
                    CommOut::PrintHistory => Shared::print_history(&board),
                    CommOut::PrintEval(eval, phase) => Shared::print_eval(eval, phase),
                    CommOut::PrintState(state) => Shared::print_state(&state),
                    CommOut::PrintHelp => Shared::print_help(CommType::XBOARD),

                    // Ignore everything else
                    _ => (),
                }
            }
        });

        // Store handle and control sender.
        self.output_handle = Some(output_handle);
        self.output_tx = Some(output_tx);
    }
}

// Implement sending/response functions
impl XBoard {
    fn new_line() {
        println!();
    }

    fn features() {
        let myname = format!("myname=\"{} {}\"", About::ENGINE, About::VERSION);

        for f in FEATURES {
            let value = f.to_string().replace("myname=x", &myname[..]);
            println!("feature {}", value);
        }
    }

    fn pong(value: i8) {
        println!("pong {}", value)
    }

    fn message(msg: &str) {
        println!("# {}", msg);
    }

    fn illegal_move(m: &str) {
        println!("Illegal move: {}", m);
    }

    fn error(err_type: &str, cmd: &str) {
        println!("Error ({}): {}", err_type, cmd);
    }

    fn best_move(m: &Move) {
        println!("move {}", m.as_string());
    }

    fn search_summary(stat01: &mut Stat01, s: &SearchSummary) {
        // This function will cache the incoming search summary within the
        // "stat01" struct that lives in the output thread.
        stat01.depth = s.depth;
        stat01.time = s.time;
        stat01.nodes = s.nodes;

        // It will also output the current search summary.
        // DEPTH SCORE TIME NODES PV
        println!(
            "{} {} {} {} {}",
            s.depth,
            s.cp,
            (s.time as f64 / 10.0).round(),
            s.nodes,
            s.pv_as_string()
        );
    }

    fn search_stats(stat01: &mut Stat01, s: &SearchStats) {
        // Update the cached search stats with new time information.
        stat01.time = s.time;
        stat01.nodes = s.nodes;
    }

    fn search_current_move(stat01: &mut Stat01, scm: &SearchCurrentMove) {
        // Update cached search stats with new move information.s
        stat01.curr_move = scm.curr_move;
        stat01.curr_move_number = scm.curr_move_number;
        stat01.legal_moves_total = scm.legal_moves_total;
    }

    fn stat01(s: &Stat01) {
        // stat01: TIME NODES DEPTH MOVESLEFT TOTALMOVES CURRENTMOVE
        if s.curr_move.get_move() != 0 {
            println!(
                "stat01: {} {} {} {} {} {}",
                (s.time as f64 / 10.0).round(),
                s.nodes,
                s.depth,
                s.legal_moves_total - s.curr_move_number,
                s.legal_moves_total,
                s.curr_move.as_string()
            );
        }
    }
}
