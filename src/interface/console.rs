// This is the engine's console interface. It can be compiled into the program
// as an optional feature, for playing in a console window. It can also call
// various test routines.

use crate::board::{playmove, representation::Board, Pieces, SQUARE_NAME};
use crate::defs::{Piece, Square, ENGINE};
use crate::extra::{perft, perftsuite, print};
use crate::movegen::{movedefs::Move, movelist::MoveList};
use crate::search::{self, SearchInfo};
use crate::utils::parse;
use if_chain::if_chain;
use std::{io, io::Write};

const CMD_STR_ERR_IO: &str = "Command-line i/o error";
const CMD_QUIT: u64 = 0;
const CMD_CONTINUE: u64 = 1;

// Errors when parsing moves
const ERR_MV_NO_ERROR: u8 = 0;
const ERR_MV_SQUARE_ERROR: u8 = 1;
const ERR_MV_NOT_PROMOTION: u8 = 2;
const ERR_MV_LENGTH_WRONG: u8 = 3;
const ERR_MV_STRINGS: [&str; 4] = [
    "No error.",
    "Square doesn't exist.",
    "Not a promotion piece.",
    "Move length is wrong.",
];

type ParseMoveResult = Result<(Square, Square, Piece), u8>;
type PotentialMove = (Square, Square, Piece);

// TODO: Update comment.
pub fn get_input(board: &mut Board) -> u64 {
    let mut input = String::new();

    print::horizontal_line('=', 40);
    print::position(board, None);
    print!("{} > ", ENGINE);

    match io::stdout().flush() {
        Ok(()) => (),
        Err(error) => panic!("{}: {}", CMD_STR_ERR_IO, error),
    }
    match io::stdin().read_line(&mut input) {
        Ok(_) => parse_input(board, &mut input),
        Err(error) => panic!("{}: {}", CMD_STR_ERR_IO, error),
    }
}

// TODO: Update comment.
pub fn parse_input(board: &mut Board, input: &mut String) -> u64 {
    parse::strip_newline(input);
    match &input[..] {
        "quit" | "exit" => CMD_QUIT,
        "perft" => cmd_perft(board),
        "suite" => cmd_suite(),
        "clear" => cmd_clear(),
        "t" => cmd_take_move(board),
        "m" => cmd_search(board),
        _ => cmd_make_move(board, input),
    }
}

// TODO: Update comment.
fn cmd_search(board: &mut Board) -> u64 {
    let mut info = SearchInfo::new();
    let m: Move = search::alpha_beta(board, &mut info, 1);
    playmove::make(board, m);
    println!(
        "{} has moved: {}{}",
        ENGINE,
        SQUARE_NAME[m.from() as usize],
        SQUARE_NAME[m.to() as usize]
    );
    CMD_CONTINUE
}

// TODO: Update comment.
fn cmd_clear() -> u64 {
    CMD_CONTINUE
}

// TODO: Update comment.
fn cmd_perft(board: &Board) -> u64 {
    perft::run(board, 7);
    CMD_CONTINUE
}

// TODO: Update comment.
fn cmd_suite() -> u64 {
    perftsuite::run_all_tests();
    CMD_CONTINUE
}

// TODO: Update comment.
fn cmd_take_move(board: &mut Board) -> u64 {
    if board.history.len() >= 1 {
        playmove::unmake(board);
    }
    CMD_CONTINUE
}

// TODO: Update comment.
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

// TODO: Update comment.
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
