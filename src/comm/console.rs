// This file implements the Console interface. In this communication mode,
// the engine shows the current board position in the terminal, and it will
// accept commands typed by the user. This interface is mainly used for
// engine development.

use super::IComm;
use std::thread;
use std::thread::JoinHandle;

use super::{Command, ErrComm};
use crate::{board::Board, defs::About, misc::print};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};
pub struct Console {}

impl Console {
    pub fn new() -> Self {
        Self {}
    }

    // This function creates the engines command prompt.
    fn create_prompt() {
        const PROMPT: &str = ">";

        print!("{} {} ", About::ENGINE, PROMPT);

        // Flush the output so the prompt is actually printed.
        match io::stdout().flush() {
            Ok(()) => {}
            Err(e) => panic!("Error flushing I/O: {}", e),
        }
    }

    // This function transforms the typed characters into a command tht the
    // engine which is running in the main thread can understand.
    fn create_command(input: &String) -> Command {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match &i[..] {
            "quit" | "exit" => Command::Quit,
            _ => Command::Move(i),
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
            let mut cmd = Command::NoCmd;
            // As long as no "quit" or "exit" commands are detected, the
            // result will be 0 and the console keeps running.
            while cmd != Command::Quit {
                let mut input: String = String::from("");

                // Print a divider line, the position, and the prompt.
                println!("{}", "=".repeat(DIVIDER_LENGTH));
                print::position(&board.lock().expect(ErrComm::LOCK_BOARD), None);
                Console::create_prompt();

                // Wait for actual commands to be entered.
                io::stdin().read_line(&mut input).expect(ErrComm::READ_IO);

                // Parse the input and catch the command.
                cmd = Console::create_command(&input);
                if !cmd.is_correct() {
                    print!("Unknown command: {}", input);
                }
            }
        });

        handle
    }
}
