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

use super::{CommInput, CommOutput, CommType, IComm, XBoardInput, XBoardOutput};
use crate::{
    board::Board,
    engine::defs::{EngineOption, ErrFatal, Information},
    misc::print,
};
use crossbeam_channel::{self, Sender};
use std::{
    io::{self},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

// This struct is used to instantiate the Comm Console module.
pub struct XBoard {
    receiving_handle: Option<JoinHandle<()>>, // Thread for receiving input.
    output_handle: Option<JoinHandle<()>>,    // Thread for sending output.
    output_tx: Option<Sender<CommOutput>>,    // Actual output sender object.
}

// Public functions
impl XBoard {
    // Create a new console.
    pub fn new() -> Self {
        Self {
            receiving_handle: None,
            output_handle: None,
            output_tx: None,
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

    // This function just returns the name of the communication protocol.
    fn get_protocol_name(&self) -> &'static str {
        CommType::XBOARD
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
                let comm_received = XBoard::create_comm_received(&t_incoming_data);

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
    fn create_comm_received(input: &str) -> CommInput {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match i {
            cmd if cmd.starts_with("ping") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd == "quit" || cmd == "exit" || cmd.is_empty() => CommInput::Quit,

            // Custom commands
            cmd if cmd == "board" => CommInput::Board,
            cmd if cmd == "history" => CommInput::History,
            cmd if cmd == "eval" => CommInput::Eval,
            cmd if cmd == "help" => CommInput::Help,

            // Everything else is ignored.
            _ => CommInput::Unknown,
        }
    }

    fn parse_key_value_pair(cmd: &str) -> CommInput {
        const KEY: usize = 0;
        const VALUE: usize = 1;
        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_lowercase()).collect();

        match &parts[KEY][..] {
            "ping" => {
                let value = parts[VALUE].parse::<u8>().unwrap_or(0);
                CommInput::XBoard(XBoardInput::Ping(value))
            }

            _ => CommInput::Unknown,
        }
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
            let t_options = Arc::clone(&options);

            // Keep running as long as Quit is not received.
            while !quit {
                let output = output_rx.recv().expect(ErrFatal::CHANNEL);

                // Perform command as sent by the engine thread.
                match output {
                    CommOutput::XBoard(XBoardOutput::Pong(v)) => println!("pong {}", v),
                    CommOutput::Quit => quit = true,

                    // Custom prints for use in the console.
                    CommOutput::PrintBoard => XBoard::print_board(&t_board),
                    CommOutput::PrintHistory => XBoard::print_history(&t_board),
                    CommOutput::PrintEval(eval, phase) => XBoard::print_eval(eval, phase),
                    CommOutput::PrintHelp => XBoard::print_help(),

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
impl XBoard {}

// implements handling of custom commands. These are mostly used when using
// the UCI protocol directly in a terminal window.
impl XBoard {
    fn print_board(board: &Arc<Mutex<Board>>) {
        print::position(&board.lock().expect(ErrFatal::LOCK), None);
    }

    fn print_history(board: &Arc<Mutex<Board>>) {
        let mtx_board = board.lock().expect(ErrFatal::LOCK);
        let length = mtx_board.history.len();

        if length == 0 {
            println!("No history available.");
        }

        for i in 0..length {
            let h = mtx_board.history.get_ref(i);
            println!("{:<3}| ply: {} {}", i, i + 1, h.as_string());
        }

        std::mem::drop(mtx_board);
    }

    fn print_eval(eval: i16, phase: i16) {
        println!("Evaluation: {}, Phase: {}", eval, phase);
    }

    fn print_help() {
        println!("The engine is in XBoard communication mode. It supports some custom");
        println!("non-UCI commands to make use through a terminal window easier.");
        println!("These commands can also be very useful for debugging purposes.");
        println!();
        println!("Custom commands");
        println!("================================================================");
        println!("help      :   This help information.");
        println!("board     :   Print the current board state.");
        println!("history   :   Print a list of past board states.");
        println!("eval      :   Print evaluation for side to move.");
        println!("exit      :   Quit/Exit the engine.");
        println!();
    }
}
