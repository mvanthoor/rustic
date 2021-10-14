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

use super::{shared::Shared, CommInput, CommOutput, CommType, IComm};
use crate::{
    board::Board,
    defs::About,
    engine::defs::{EngineOption, ErrFatal, Information},
    movegen::defs::Move,
    search::defs::SearchSummary,
};
use crossbeam_channel::{self, Sender};
use std::{
    io::{self},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

const PROTOCOL_NAME: &str = "XBoard";
const BASIC_FEATURES: [&str; 8] = [
    "ping=1",
    "memory=1",
    "setboard=1",
    "debug=1",
    "usermove=1",
    "draw=0",
    "sigint=0",
    "sigterm=0",
];
const KEY: usize = 0;
const VALUE: usize = 1;
const MILLISECONDS: u128 = 1000;

// This struct is used to instantiate the Comm Console module.
pub struct XBoard {
    receiving_handle: Option<JoinHandle<()>>, // Thread for receiving input.
    output_handle: Option<JoinHandle<()>>,    // Thread for sending output.
    output_tx: Option<Sender<CommOutput>>,    // Actual output sender object.
    time: Arc<Mutex<XBoardTime>>,             // Time management settings.
}

#[derive(PartialEq, Clone)]
pub enum XBoardInput {
    XBoard,
    ProtoVer(u8),
    New,
    Force,
    SetBoard(String),
    UserMove(String, XBoardTime),
    Go,
    MoveNow,
    Ping(i8),
    Memory(usize),
    Analyze,
    Exit,
}

pub enum XBoardOutput {
    NewLine,
    Features,
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
            time: Arc::new(Mutex::new(XBoardTime::new())),
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct XBoardTime {
    pub sd: i8,
    pub st: u128,
    pub moves_per_session: u8,
    pub basetime: u64,
    pub increment: u64,
}

impl XBoardTime {
    fn new() -> Self {
        Self {
            sd: 0,
            st: 0,
            moves_per_session: 0,
            basetime: 0,
            increment: 0,
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
        self.receiving_thread(receiving_tx);
        self.output_thread(board, options);
    }

    // The engine thread (which is the creator of the Comm module) can use
    // this function to send out of the engine onto the console, or towards
    // a user interface.
    fn send(&self, msg: CommOutput) {
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

    fn info(&self) -> CommType {
        CommType {
            protocol: String::from(PROTOCOL_NAME),
            stateful: true,
            fancy_about: false,
        }
    }
}

// Implement the receiving thread
impl XBoard {
    // The receiving thread receives incoming commands from the console or
    // GUI, which is turns into a "CommInput" object. It sends this
    // object to the engine thread so the engine can decide what to do.
    fn receiving_thread(&mut self, receiving_tx: Sender<Information>) {
        // Create thread-local variables
        let mut t_incoming_data = String::from(""); // Buffer for incoming data.
        let t_receiving_tx = receiving_tx; // Sends incoming data to engine thread.
        let t_time = Arc::clone(&self.time);

        // Actual thread creation.
        let receiving_handle = thread::spawn(move || {
            let mut quit = false;

            // Keep running as long as 'quit' is not detected.
            while !quit {
                // Get data from stdin.
                io::stdin()
                    .read_line(&mut t_incoming_data)
                    .expect(ErrFatal::READ_IO);

                // Create the CommInput object.
                let comm_received = XBoard::create_comm_received(&t_incoming_data, &t_time);

                // Send it to the engine thread.
                t_receiving_tx
                    .send(Information::Comm(comm_received.clone()))
                    .expect(ErrFatal::HANDLE);

                // Terminate the receiving thread if "Quit" was detected.
                quit = comm_received == CommInput::Quit;

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
    // This function turns the incoming data into CommInputs which the
    // engine is able to understand and react to.
    fn create_comm_received(input: &str, time: &Arc<Mutex<XBoardTime>>) -> CommInput {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match i {
            cmd if cmd == "xboard" => CommInput::XBoard(XBoardInput::XBoard),
            cmd if cmd == "new" => CommInput::XBoard(XBoardInput::New),
            cmd if cmd == "force" => CommInput::XBoard(XBoardInput::Force),
            cmd if cmd == "go" => CommInput::XBoard(XBoardInput::Go),
            cmd if cmd == "?" => CommInput::XBoard(XBoardInput::MoveNow),
            cmd if cmd == "analyze" => CommInput::XBoard(XBoardInput::Analyze),
            cmd if cmd == "exit" => CommInput::XBoard(XBoardInput::Exit),
            cmd if cmd.starts_with("ping") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("protover") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("memory") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("setboard") => XBoard::parse_setboard(&cmd),
            cmd if cmd.starts_with("usermove") => XBoard::parse_usermove(&cmd, time),
            cmd if cmd.starts_with("sd") => XBoard::parse_time(&cmd, time),
            cmd if cmd.starts_with("st") => XBoard::parse_time(&cmd, time),
            cmd if cmd == "quit" || cmd.is_empty() => CommInput::Quit,

            // Custom commands
            cmd if cmd == "board" => CommInput::Board,
            cmd if cmd == "history" => CommInput::History,
            cmd if cmd == "eval" => CommInput::Eval,
            cmd if cmd == "help" => CommInput::Help,

            // No specific command; try to parse as move.
            _ => XBoard::parse_move(&i, time),
        }
    }

    fn parse_key_value_pair(cmd: &str) -> CommInput {
        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();

        // Key-value pair has to have two parts. Ignore anything else after
        // the second part.
        if parts.len() >= 2 {
            match &parts[KEY][..] {
                "ping" => {
                    let value = parts[VALUE].parse::<i8>().unwrap_or(0);
                    CommInput::XBoard(XBoardInput::Ping(value))
                }
                "protover" => {
                    let value = parts[VALUE].parse::<u8>().unwrap_or(0);
                    CommInput::XBoard(XBoardInput::ProtoVer(value))
                }
                "memory" => {
                    let value = parts[VALUE].parse::<usize>().unwrap_or(0);
                    CommInput::XBoard(XBoardInput::Memory(value))
                }
                _ => CommInput::Unknown,
            }
        } else {
            CommInput::Unknown
        }
    }

    fn parse_usermove(cmd: &str, time: &Arc<Mutex<XBoardTime>>) -> CommInput {
        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
        CommInput::XBoard(XBoardInput::UserMove(
            parts[VALUE].to_lowercase(),
            time.lock().expect(ErrFatal::LOCK).clone(),
        ))
    }

    fn parse_time(cmd: &str, time: &Arc<Mutex<XBoardTime>>) -> CommInput {
        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
        let mut result = CommInput::Ok;
        let mut mtx_time = time.lock().expect(ErrFatal::LOCK);

        if parts.len() >= 2 {
            match &parts[KEY][..] {
                "sd" => mtx_time.sd = parts[VALUE].parse::<i8>().unwrap_or(0),
                "st" => {
                    // Cancel settings from level command.
                    mtx_time.basetime = 0;
                    mtx_time.increment = 0;
                    mtx_time.moves_per_session = 0;

                    // Set move time in milliseconds.
                    mtx_time.st = parts[VALUE].parse::<u128>().unwrap_or(0) * MILLISECONDS;
                }
                _ => result = CommInput::Unknown,
            }
        } else {
            result = CommInput::Unknown;
        }

        std::mem::drop(mtx_time);

        result
    }

    fn parse_setboard(cmd: &str) -> CommInput {
        let fen = cmd.replace("setboard", "").trim().to_string();
        CommInput::XBoard(XBoardInput::SetBoard(fen))
    }

    fn parse_move(cmd: &str, time: &Arc<Mutex<XBoardTime>>) -> CommInput {
        const COORD_ALPHA: &str = "abcdefgh";
        const COORD_DIGIT: &str = "12345678";
        const PROMOTION: &str = "qrbn";

        let input = cmd.to_lowercase();
        let mut chars_ok: u8 = 0;
        let mut result = CommInput::Unknown;

        // Now check if the input can actually be a move.
        if input.len() == 4 || input.len() == 5 {
            for (i, char) in input.chars().enumerate() {
                match i {
                    0 | 2 if COORD_ALPHA.contains(char) => chars_ok += 1,
                    1 | 3 if COORD_DIGIT.contains(char) => chars_ok += 1,
                    4 if PROMOTION.contains(char) => chars_ok += 1,
                    _ => (),
                }
            }
        }

        // If all characters are within specified ranges, then we have a
        // possible move. Send this to the engine.
        if chars_ok == (input.len() as u8) {
            let t = time.lock().expect(ErrFatal::LOCK).clone();
            result = CommInput::XBoard(XBoardInput::UserMove(input, t));
        }

        result
    }
}

// Implement the output thread
impl XBoard {
    // The control thread receives commands from the engine thread.
    fn output_thread(&mut self, board: Arc<Mutex<Board>>, options: Arc<Vec<EngineOption>>) {
        // Create an incoming channel for the control thread.
        let (output_tx, output_rx) = crossbeam_channel::unbounded::<CommOutput>();

        // Create the output thread.
        let output_handle = thread::spawn(move || {
            let mut quit = false;
            let t_board = Arc::clone(&board);

            // Engine options are not used by XBoard at this point.
            let _t_options = Arc::clone(&options);

            // Keep running as long as Quit is not received.
            while !quit {
                let output = output_rx.recv().expect(ErrFatal::CHANNEL);

                // Perform command as sent by the engine thread.
                match output {
                    CommOutput::XBoard(XBoardOutput::NewLine) => XBoard::new_line(),
                    CommOutput::XBoard(XBoardOutput::Features) => XBoard::features(),
                    CommOutput::XBoard(XBoardOutput::Pong(v)) => XBoard::pong(v),
                    CommOutput::XBoard(XBoardOutput::IllegalMove(m)) => XBoard::illegal_move(m),
                    CommOutput::SearchSummary(summary) => XBoard::search_summary(&summary),
                    CommOutput::Message(msg) => XBoard::message(msg),
                    CommOutput::BestMove(m) => XBoard::best_move(m),
                    CommOutput::Quit => quit = true,

                    // Custom prints for use in the console.
                    CommOutput::PrintBoard => Shared::print_board(&t_board),
                    CommOutput::PrintHistory => Shared::print_history(&t_board),
                    CommOutput::PrintEval(eval, phase) => Shared::print_eval(eval, phase),
                    CommOutput::PrintHelp => Shared::print_help(PROTOCOL_NAME),

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
        println!("feature done=0");
        println!("feature myname=\"{} {}\"", About::ENGINE, About::VERSION);

        for f in BASIC_FEATURES {
            println!("feature {}", f);
        }

        println!("feature done=1");
    }

    fn pong(value: i8) {
        println!("pong {}", value)
    }

    fn message(msg: String) {
        println!("{}", msg);
    }

    fn best_move(m: Move) {
        println!("move {}", m.as_string());
    }

    fn search_summary(s: &SearchSummary) {
        println!(
            "{} {} {} {} {}",
            s.depth,
            s.cp,
            (s.time as f64 / 10.0).round(),
            s.nodes,
            s.pv_as_string()
        );
    }

    fn illegal_move(m: String) {
        println!("Illegal move: {}", m);
    }
}
