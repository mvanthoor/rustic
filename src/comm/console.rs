// This file implements the Console interface. In this communication mode,
// the engine shows the current board position in the terminal, and it will
// accept commands typed by the user. This interface is mainly used for
// engine development.

use super::{CommControl, CommReport, CommType, ErrFatal, IComm};
use crate::{board::Board, defs::About, misc::print};
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

// Any communication module must implement the trait IComm.
impl IComm for Console {
    fn activate(
        &mut self,
        report_tx: Sender<CommReport>,
        board: Arc<Mutex<Board>>,
    ) -> Sender<CommControl> {
        println!("Starting Comm Reader thread.");

        // Create the reader thread for reading data from the UI/Stdin.
        let h_reader = thread::spawn(move || {
            // The default is to do nothing.
            let mut report = CommReport::Nothing;

            // Keep running as long as 'quit' is not detected.
            while report != CommReport::Quit {
                // Get data from stdin and create a report from it.
                let mut input: String = String::from("");
                io::stdin().read_line(&mut input).expect(ErrFatal::READ_IO);
                report = Console::create_report(&input);

                report_tx
                    .send(report.clone())
                    .expect(ErrFatal::BROKEN_HANDLE);

                if !report.is_valid() {
                    println!("Unknown input: {}", input);
                }
            }

            println!("Quitting Comm Reader thread.");
        });
        self.handle_reader = Some(h_reader);

        // Create the thread for control commands from the engine.
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<CommControl>();
        let h_control = thread::spawn(move || {
            println!("Starting Comm Control thread.");

            let mut control = CommControl::Nothing;
            while control != CommControl::Quit {
                control = control_rx.recv().expect(ErrFatal::BROKEN_HANDLE);

                match control {
                    CommControl::Quit => (),
                    CommControl::Update => Console::print_position(board.clone()),
                    _ => (),
                }
            }

            println!("Quitting Comm Control thread.");
        });
        self.handle_control = Some(h_control);

        control_tx
    }

    fn wait_for_shutdown(&mut self) {
        println!("Waiting for Comm shutdown...");
        if let Some(h) = self.handle_reader.take() {
            h.join().expect(ErrFatal::FAILED_THREAD);
        }

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
