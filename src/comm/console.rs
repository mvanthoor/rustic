// This file implements the Console interface. In this communication mode,
// the engine shows the current board position in the terminal, and it will
// accept commands typed by the user. This interface is mainly used for
// engine development.

use super::IComm;
use std::thread;
use std::thread::JoinHandle;

use super::{ErrFatal, Incoming};
use crate::{board::Board, defs::About, misc::print};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};
pub struct Console {}

impl Console {
    const PROMPT: &'static str = ">";
    pub const UNKNOWN_COMMAND: &'static str = "Unknown command";

    pub fn new() -> Self {
        Self {}
    }

    // This function creates the engine's command prompt.
    fn create_prompt() {
        print!("{} {} ", About::ENGINE, Console::PROMPT);
        io::stdout().flush().expect(ErrFatal::FLUSH_IO);
    }

    // This function transforms the typed characters into a command tht the
    // engine which is running in the main thread can understand.
    fn create_command(input: &String) -> Incoming {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match &i[..] {
            "quit" | "exit" => Incoming::Quit,
            _ => Incoming::Move(i),
        }
    }
}

// Any communication module must implement the trait IComm.
impl IComm for Console {
    // This function starts the communication thread.
    fn start(&self, board: Arc<Mutex<Board>>) -> JoinHandle<()> {
        const DIVIDER_LENGTH: usize = 48;

        // Run the communication in its own thread.
        let handle = thread::spawn(move || {
            let mut cmd = Incoming::NoCmd;
            // As long as no "quit" or "exit" commands are detected, the
            // result will be 0 and the console keeps running.
            while cmd != Incoming::Quit {
                let mut input: String = String::from("");

                // Print a divider line, the position, and the prompt.
                println!("{}", "=".repeat(DIVIDER_LENGTH));
                let mtx_board = board.lock().expect(ErrFatal::LOCK_BOARD);
                print::position(&mtx_board, None); // Print the board.
                std::mem::drop(mtx_board); // Drop the lock: no longer needed.
                Console::create_prompt();

                // Wait for actual commands to be entered.
                io::stdin().read_line(&mut input).expect(ErrFatal::READ_IO);

                // Parse the input and catch the command.
                cmd = Console::create_command(&input);
                if !cmd.is_correct() {
                    print!("{}: {}", Console::UNKNOWN_COMMAND, input);
                }
            }
        });

        handle
    }
}
