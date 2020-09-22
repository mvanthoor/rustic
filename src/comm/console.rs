// This file implements the Console interface. In this communication mode,
// the engine shows the current board position in the terminal, and it will
// accept commands typed by the user. This interface is mainly used for
// engine development.

use super::{ErrFatal, IComm, Incoming, IncomingRx};
use crate::{board::Board, defs::About, misc::print};
use std::{
    io::{self, Write},
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};
pub struct Console {
    handle: Option<JoinHandle<()>>,
}

impl Console {
    const DIVIDER_LENGTH: usize = 48;
    const PROMPT: &'static str = ">";

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
    fn read(&mut self, board: Arc<Mutex<Board>>) -> IncomingRx {
        // Create a sender (tx) and receiver (rx) so this thread can send
        // incoming commands from the tx to the rx part. rx will be
        // returned to the caller of read().
        let (tx, rx) = mpsc::channel::<Incoming>();

        // Run the comm reader in its own thread. This is necessary,
        // because read_line would block the main engine thread.
        let h = thread::spawn(move || {
            // No Command is the default while waiting for input.
            let mut cmd = Incoming::NoCmd;

            // Keep running until Quit command detected.
            while cmd != Incoming::Quit {
                let mut input: String = String::from("");

                // Print a divider line between the about message and board.
                println!("{}", "=".repeat(Console::DIVIDER_LENGTH));

                // Lock the board using a mutex.
                let mtx_board = board.lock().expect(ErrFatal::LOCK_BOARD);

                // Print the position.
                print::position(&mtx_board, None); // Print the board.

                // Mutix is no longer needed. Drop it to unlock the board.
                std::mem::drop(mtx_board);

                // Print the console prompt.
                Console::create_prompt();

                // Wait for actual commands to be entered.
                io::stdin().read_line(&mut input).expect(ErrFatal::READ_IO);

                // Parse the input and transform it into a command.
                cmd = Console::create_command(&input);

                // If the result is a legal command, send it to the rx part.
                if cmd.is_correct() {
                    tx.send(cmd.clone()).expect(ErrFatal::CHANNEL_BROKEN);
                }
            }
        });

        // Store the generated handle.
        self.handle = Some(h);

        // Return receiver part to caller.
        rx
    }

    // This function will return the thread handle, so the caller can join
    // on it and know that this thread has ended.
    fn get_thread_handle(&mut self) -> Option<JoinHandle<()>> {
        self.handle.take()
    }
}
