use super::IComm;
use std::thread;
use std::thread::JoinHandle;

use crate::{board::Board, defs::About, misc::print};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

// type ParseMoveResult = Result<(Square, Square, Piece), u8>;
// type PotentialMove = (Square, Square, Piece);

/*
#[derive(PartialEq)]
enum CommState {
    Quit,
    Continue,
}
*/

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
            let mut result = 0;

            // As long as no "quit" or "exit" commands are detected, the
            // result will be 0 and the console keeps running.
            while result == 0 {
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

                // Parse the typed command and catch the result.
                result = parse_input(input.trim_end().to_string());
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
fn parse_input(input: String) -> u8 {
    let mut result = 0;

    match &input[..] {
        "quit" | "exit" => result = 1,
        _ => {}
    }

    result
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

fn parse_move(input: &str) -> ParseMoveResult {
    let length = input.len();
    let mut from: Square = 0;
    let mut to: Square = 0;
    let mut promotion_piece: Piece = Pieces::NONE;
    let mut result: ParseMoveResult = Err(ERR_MV_NO_ERROR);

    // Check if chars 1-2 and 3-4 are actually represent squares.
    if length == 4 || length == 5 {
        if_chain! {
            if let Ok(f) = parse::algebraic_square_to_number(&input[0..=1]);
            if let Ok(t) = parse::algebraic_square_to_number(&input[2..=3]);
            then {
                from = f;
                to = t;
            } else {
                result = Err(ERR_MV_SQUARE_ERROR);
            }
        };
    }

    // If there's a fifth character, check if it's a legal promotion piece.
    if length == 5 {
        if_chain! {
            if let Some(c) = input.chars().next_back();
            if let Ok(p) = parse::promotion_piece_letter_to_number(c);
            then {
                promotion_piece = p;
            } else {
                result = Err(ERR_MV_NOT_PROMOTION);
            }
        }
    }

    // Input is of wrong length. Fail; don't check anything.
    if (length != 4) && (length != 5) {
        result = Err(ERR_MV_LENGTH_WRONG);
    }

    // If there is no error, then result becomes Ok(); this is a potential move.
    if result == Err(ERR_MV_NO_ERROR) {
        result = Ok((from, to, promotion_piece));
    }
    result
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
