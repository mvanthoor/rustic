use super::IComm;
use std::thread;
use std::thread::JoinHandle;

use crate::{
    board::Board,
    defs::About,
    misc::{parse, print},
};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

#[derive(PartialEq)]
enum Command {
    NoCmd,
    Quit,
    Move(String),
}

// This file implements the Console interface. In this mode, the engine
// shows the current board position. It will accept a few commands to
// change the engine settings, and it will accept move inputs in the form
// of "g1f3" or "b7b8q". In this way a game can be played. The console is
// mainly intended for developers to test new functionality.
pub struct Console {}

impl Console {
    pub fn new() -> Self {
        Self {}
    }
}

// Any communication module must implement the trait IComm.
impl IComm for Console {
    // This function starts the communication. In this case, the
    // communication is through the console, so the user can input commands
    // and moves directly.
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
                print::position(&board.lock().unwrap(), None);
                create_prompt();

                // Wait for actual commands to be entered.
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {}
                    Err(e) => panic!("Error reading I/O: {}", e),
                }

                // Parse the input and catch the command.
                cmd = parse_input(input.trim_end().to_string());
                execute_command(&cmd);
            }
        });

        handle
    }
}

// This function creates Rustic's command prompt
fn create_prompt() {
    const PROMPT: &str = ">";

    print!("{} {} ", About::ENGINE, PROMPT);

    // Flush so the prompt is actually printed.
    match io::stdout().flush() {
        Ok(()) => {}
        Err(e) => panic!("Error flushing I/O: {}", e),
    }
}

// Parse the entered commands and return the results.
fn parse_input(input: String) -> Command {
    match &input[..] {
        "quit" | "exit" => Command::Quit,
        _ => Command::Move(input),
    }
}

fn execute_command(cmd: &Command) {
    match cmd {
        Command::NoCmd | Command::Quit => (),
        Command::Move(m) => {
            match parse::algebraic_move_to_number(&m[..]) {
                Ok(m) => println!("Whieee! {}-{}-{}", m.0, m.1, m.2),
                Err(()) => println!("Oops..."),
            };
        }
    }
}

/*

fn cmd_make_move(board: &mut Board, input: &str) -> u64 {
    let parse_move_result = parse_move(input);
    let mut try_move_result = Err(());
    match parse_move_result {
        Ok(potential_move) => try_move_result = try_move(board, potential_move),
        Err(e) => println!("Parsing error: {}", ERR_MV_STRINGS[e as usize]),
    }
    match try_move_result {
        Ok(()) => println!("Player has moved."),
        Err(()) if parse_move_result.is_ok() => println!("Illegal move."),
        Err(_) => (),
    }
    CMD_CONTINUE
}

// This function can be used to try and play the move resulting from parse_move().
fn try_move(board: &mut Board, potential_move: PotentialMove) -> Result<(), ()> {
    let mut move_list = MoveList::new();
    let mut result = Err(());

    board.gen_all_moves(&mut move_list);
    for i in 0..move_list.len() {
        let current_move = move_list.get_move(i);
        if_chain! {
            if potential_move.0 == current_move.from();
            if potential_move.1 == current_move.to();
            if potential_move.2 == current_move.promoted();
            if playmove::make(board, current_move);
            then {
                result = Ok(());
                break;
            }
        }
    }
    result
}

*/
