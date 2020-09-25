// This file implements the Console interface. In this communication mode,
// the engine shows the current board position in the terminal, and it will
// accept commands typed by the user. This interface is mainly used for
// engine development.

use super::{CommReport, CommType, ErrFatal, IComm};
use crate::{board::Board, defs::About, misc::print};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};
pub struct Console {}

impl Console {
    const DIVIDER_LENGTH: usize = 48;
    const PROMPT: &'static str = ">";

    pub fn new() -> Self {
        Self {}
    }

    // This function creates the engine's command prompt.
    fn print_prompt() {
        print!("{} {} ", About::ENGINE, Console::PROMPT);
        io::stdout().flush().expect(ErrFatal::FLUSH_IO);
    }

    // This function transforms the typed characters into a command tht the
    // engine which is running in the main thread can understand.
    fn create_command(input: &String) -> CommReport {
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
    // Some protocols require output before reading; in the case of
    // "console", the board position and prompt must be printed.
    fn print_before_read(&self, board: Arc<Mutex<Board>>) {
        let mtx_board = board.lock().expect(ErrFatal::LOCK_BOARD); // Lock the board.
        println!("{}", "=".repeat(Console::DIVIDER_LENGTH)); // Print divider.
        print::position(&mtx_board, None); // Print board position.
        std::mem::drop(mtx_board); // Drop the mutex and unlock the board.
        Console::print_prompt(); // Print the command prompt.
    }

    // This function reads commands from the console (keyboard)
    fn read(&self) -> CommReport {
        let mut input: String = String::from("");

        // Read text from stdin
        io::stdin().read_line(&mut input).expect(ErrFatal::READ_IO);

        // Parse to command and verify correctness
        let cmd = Console::create_command(&input);
        if cmd.is_correct() {
            cmd
        } else {
            CommReport::Nothing
        }
    }

    // This function just returns the name of the communication protocol.
    fn get_protocol_name(&self) -> &'static str {
        CommType::CONSOLE
    }
}
