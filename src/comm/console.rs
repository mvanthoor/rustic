// This file implements the Console interface. In this communication mode,
// the engine shows the current board position in the terminal, and it will
// accept commands typed by the user. This interface is mainly used for
// engine development.

use super::{CommControl, CommReport, CommType, IComm};
use crate::{board::Board, defs::About, engine::ErrFatal, engine::Information, misc::print};
use crossbeam_channel::{self, Sender};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct Console {
    control_handle: Option<JoinHandle<()>>,
    reader_handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<CommControl>>,
}

// Any communication module must implement the trait IComm.
impl IComm for Console {
    fn activate(&mut self, report_tx: Sender<Information>, board: Arc<Mutex<Board>>) {
        println!("Starting Comm Reader thread.");

        // Create thread-local variables.
        let t_reader_board = board.clone();
        let mut t_input = String::from("");

        // Create the reader thread for reading data from the UI/Stdin.
        let reader_handle = thread::spawn(move || {
            let mut quit = false;

            // Keep running as long as 'quit' is not detected.
            while !quit {
                // Get data from stdin and create a report from it.
                io::stdin()
                    .read_line(&mut t_input)
                    .expect(ErrFatal::READ_IO);
                let report = Console::create_report(&t_input);

                // Terminate at the end of this iteration.
                quit = report == CommReport::Quit;

                // If the report is valid, send it as comm information.
                if report.is_valid() {
                    let information = Information::Comm(report);
                    report_tx.send(information).expect(ErrFatal::HANDLE);
                } else {
                    // Or give an error message, and print the board again.
                    println!("Unknown input: {}", t_input);
                    Console::print_position(t_reader_board.clone());
                }

                // Clear input for next iteration.
                t_input = String::from("");
            }

            println!("Quitting Comm Reader thread.");
        });

        // Store the handle.
        self.reader_handle = Some(reader_handle);

        // ============================================================================

        // Create a channel for the Writer thread.
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<CommControl>();

        // Create the control thread.
        let control_handle = thread::spawn(move || {
            println!("Starting Comm Control thread.");

            let mut quit = false;

            while !quit {
                let control = control_rx.recv().expect(ErrFatal::CHANNEL);

                match control {
                    CommControl::Quit => quit = true,
                    CommControl::Update => Console::print_position(Arc::clone(&board)),
                    CommControl::Write(msg) => println!("{}", msg),
                }
            }

            println!("Quitting Comm Control thread.");
        });

        // Store handle and control sender.
        self.control_handle = Some(control_handle);
        self.control_tx = Some(control_tx);
    }

    // Send messages to the control thread.
    fn send(&self, msg: CommControl) {
        if let Some(tx) = self.control_tx.clone() {
            tx.send(msg).expect(ErrFatal::CHANNEL);
        }
    }

    // After the engine has send 'quit' to the control thread, it will call
    // wait_for_shutdown() and then wait here until shutdown is complete.
    fn wait_for_shutdown(&mut self) {
        println!("Waiting for Comm shutdown...");

        // Shutting down reader thread.
        if let Some(h) = self.reader_handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }

        // Shutting down control thread.
        if let Some(h) = self.control_handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }

        println!("Comm shutdown completed.");
    }

    // This function just returns the name of the communication protocol.
    fn get_protocol_name(&self) -> &'static str {
        CommType::CONSOLE
    }
}

// Public functions
impl Console {
    pub fn new() -> Self {
        Self {
            control_handle: None,
            reader_handle: None,
            control_tx: None,
        }
    }
}

// Private functions for this module.
impl Console {
    const DIVIDER_LENGTH: usize = 48;
    const PROMPT: &'static str = ">";

    // This function creates the engine's command prompt.
    fn print_prompt() {
        print!("{} {} ", About::ENGINE, Console::PROMPT);
        io::stdout().flush().expect(ErrFatal::FLUSH_IO);
    }

    // Some protocols require output before reading; in the case of
    // "console", the board position and prompt must be printed.
    fn print_position(board: Arc<Mutex<Board>>) {
        let mtx_board = board.lock().expect(ErrFatal::LOCK);

        println!("{}", "=".repeat(Console::DIVIDER_LENGTH));
        print::position(&mtx_board, None);
        std::mem::drop(mtx_board);
        Console::print_prompt();
    }

    // This function transforms the typed characters into a command tht the
    // engine which is running in the main thread can understand.
    fn create_report(input: &str) -> CommReport {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match &i[..] {
            "quit" | "exit" => CommReport::Quit,
            "start" => CommReport::Start,
            "stop" => CommReport::Stop,
            "e" => CommReport::Evaluate,
            _ => CommReport::Move(i),
        }
    }
}
