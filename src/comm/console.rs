// This file implements the Console interface. In this communication mode,
// the engine shows the current board position in the terminal, and it will
// accept commands typed by the user. This interface is mainly used for
// engine development.

use super::{CommControl, CommReport, CommType, ErrFatal, IComm};
use crate::{board::Board, defs::About, engine::Information, misc::print};
use crossbeam_channel::{self, Sender};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct Console {
    handle_control: Option<JoinHandle<()>>,
    handle_reader: Option<JoinHandle<()>>,
}

// Any communication module must implement the trait IComm.
impl IComm for Console {
    fn activate(
        &mut self,
        report_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
    ) -> Sender<CommControl> {
        println!("Starting Comm Reader thread.");

        // Create local board for this specific thread.
        let reader_board = board.clone();

        // Create the reader thread for reading data from the UI/Stdin.
        let h_reader = thread::spawn(move || {
            let mut running = true;

            // Keep running as long as 'quit' is not detected.
            while running {
                // Get data from stdin and create a report from it.
                let mut input: String = String::from("");
                io::stdin().read_line(&mut input).expect(ErrFatal::READ_IO);
                let report = Console::create_report(&input);

                // Terminate at the end of this iteration.
                if report == CommReport::Quit {
                    running = false;
                }

                // If the report is valid, send it as comm information.
                if report.is_valid() {
                    let information = Information::Comm(report);
                    report_tx.send(information).expect(ErrFatal::BROKEN_HANDLE);
                } else {
                    // Or give an error message, and print the board again.
                    println!("Unknown input: {}", input);
                    Console::print_position(reader_board.clone());
                }
            }

            println!("Quitting Comm Reader thread.");
        });

        // Store the handle.
        self.handle_reader = Some(h_reader);

        // ============================================================================

        // Create the channel for control commands from the engine.
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<CommControl>();

        // Create local board variable for the control thread (to use in 'update').
        let control_board = board.clone();

        // Create the control thread.
        let h_control = thread::spawn(move || {
            println!("Starting Comm Control thread.");

            let mut running = true;
            while running {
                let control = control_rx.recv().expect(ErrFatal::BROKEN_HANDLE);

                match control {
                    CommControl::Quit => running = false,
                    CommControl::Update => Console::print_position(control_board.clone()),
                    CommControl::Write(msg) => println!("{}", msg),
                }
            }

            println!("Quitting Comm Control thread.");
        });

        // Store the handle.
        self.handle_control = Some(h_control);

        // Return sender for control commands.
        control_tx
    }

    // After the engine has send 'quit' to the control thread, it will call
    // wait_for_shutdown() and then wait here until shutdown is complete.
    fn wait_for_shutdown(&mut self) {
        println!("Waiting for Comm shutdown...");

        // Shutting down reader thread.
        if let Some(h) = self.handle_reader.take() {
            h.join().expect(ErrFatal::FAILED_THREAD);
        }

        // Shutting down control thread.
        if let Some(h) = self.handle_control.take() {
            h.join().expect(ErrFatal::FAILED_THREAD);
        }

        println!("Comm shutdown completed.");
    }

    // This function just returns the name of the communication protocol.
    fn get_protocol_name(&self) -> &'static str {
        CommType::CONSOLE
    }
}

impl Console {
    const DIVIDER_LENGTH: usize = 48;
    const PROMPT: &'static str = ">";

    pub fn new() -> Self {
        Self {
            handle_control: None,
            handle_reader: None,
        }
    }

    // This function creates the engine's command prompt.
    fn print_prompt() {
        print!("{} {} ", About::ENGINE, Console::PROMPT);
        io::stdout().flush().expect(ErrFatal::FLUSH_IO);
    }

    // Some protocols require output before reading; in the case of
    // "console", the board position and prompt must be printed.
    fn print_position(board: Arc<Mutex<Board>>) {
        let mtx_board = board.lock().expect(ErrFatal::LOCK_BOARD); // Lock the board.
        println!("{}", "=".repeat(Console::DIVIDER_LENGTH)); // Print divider.
        print::position(&mtx_board, None); // Print board position.
        std::mem::drop(mtx_board); // Drop the mutex and unlock the board.
        Console::print_prompt(); // Print the command prompt.
    }

    // This function transforms the typed characters into a command tht the
    // engine which is running in the main thread can understand.
    fn create_report(input: &String) -> CommReport {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match &i[..] {
            "quit" | "exit" => CommReport::Quit,
            "search" => CommReport::Search,
            _ => CommReport::Move(i),
        }
    }
}
