// This file implements the Console interface. In this communication mode,
// the engine shows the current board position in the terminal, and it will
// accept commands typed by the user. This interface is mainly used for
// engine development.

use super::{ErrFatal, IComm, Incoming};
use crate::{board::Board, defs::About, misc::print};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};
pub struct Console {
    handle: Option<JoinHandle<()>>,
}

impl Console {
    const PROMPT: &'static str = ">";
    pub const UNKNOWN_COMMAND: &'static str = "Unknown command";

    pub fn new() -> Self {
        Self { handle: None }
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
    fn start(&mut self, board: Arc<Mutex<Board>>) {
        const DIVIDER_LENGTH: usize = 48;

        // Run the communication in its own thread.
        let h = thread::spawn(move || {
            let mut cmd = Incoming::NoCmd;

            // Keep running until Quit command detected.
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

        self.handle = Some(h);
    }

    fn get_thread_handle(&mut self) -> Option<JoinHandle<()>> {
        self.handle.take()
    }
}
