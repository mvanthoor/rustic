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
    engine::defs::{EngineOption, ErrFatal, Information},
    search::defs::SearchSummary,
};
use crossbeam_channel::{self, Sender};
use std::{
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
    SetBoard(String),
    UserMove(String),
    Ping(i8),
    Post,
    NoPost,
    Memory(usize),
    Analyze,
    Exit,
}

pub enum XBoardOut {
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
            info: CommInfo::new(CommType::XBOARD, false),
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
        let i = input.trim_end().to_lowercase();

        // Convert to &str for matching the command.
        match i {
            cmd if cmd == "xboard" => CommIn::XBoard(XBoardIn::XBoard),
            cmd if cmd == "new" => CommIn::XBoard(XBoardIn::New),
            cmd if cmd == "post" => CommIn::XBoard(XBoardIn::Post),
            cmd if cmd == "nopost" => CommIn::XBoard(XBoardIn::NoPost),
            cmd if cmd == "analyze" => CommIn::XBoard(XBoardIn::Analyze),
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

            // Assume anything else is unknown.
            _ => CommIn::Unknown(i),
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
            let t_board = Arc::clone(&board);
            let _t_options = Arc::clone(&options);

            // Keep running as long as Quit is not received.
            while !quit {
                let output = output_rx.recv().expect(ErrFatal::CHANNEL);

                // Perform command as sent by the engine thread.
                match output {
                    CommOut::XBoard(XBoardOut::NewLine) => XBoard::new_line(),
                    CommOut::XBoard(XBoardOut::Features) => XBoard::features(),
                    CommOut::XBoard(XBoardOut::Pong(v)) => XBoard::pong(v),
                    CommOut::XBoard(XBoardOut::IllegalMove(m)) => XBoard::illegal_move(&m),
                    CommOut::SearchSummary(summary) => XBoard::search_summary(&summary),
                    CommOut::Message(msg) => XBoard::send_message(&msg),
                    CommOut::Quit => quit = true,

                    // Custom prints for use in the console.
                    CommOut::PrintBoard => Shared::print_board(&t_board),
                    CommOut::PrintHistory => Shared::print_history(&t_board),
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
        println!("\n");
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

    fn send_message(msg: &str) {
        println!("{}", msg);
    }

    fn illegal_move(m: &str) {
        println!("Illegal move: {}", m);
    }

    fn search_summary(s: &SearchSummary) {
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
}
