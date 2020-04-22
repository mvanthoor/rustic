// TODO: Update comments

use crate::board::{playmove, representation::Board};
use crate::defs::{Piece, ENGINE, PNONE};
use crate::extra::{perft, perftsuite, print};
use crate::movegen::movelist::MoveList;
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

type ParseMoveResult = Result<(u8, u8, u8), u8>;
type PotentialMove = (u8, u8, u8);

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

pub fn parse_input(board: &mut Board, input: &mut String) -> u64 {
    parse::strip_newline(input);
    match &input[..] {
        "quit" | "exit" => CMD_QUIT,
        "perft" => cmd_perft(board),
        "suite" => cmd_suite(),
        "clear" => cmd_clear(),
        "t" => cmd_take_move(board),
        _ => cmd_make_move(board, input),
    }
}

fn cmd_clear() -> u64 {
    CMD_CONTINUE
}

fn cmd_perft(board: &Board) -> u64 {
    perft::run(board, 7);
    CMD_CONTINUE
}

fn cmd_suite() -> u64 {
    perftsuite::run_all_tests();
    CMD_CONTINUE
}

fn cmd_take_move(board: &mut Board) -> u64 {
    if board.history.len() >= 1 {
        playmove::unmake(board);
    }
    CMD_CONTINUE
}

fn cmd_make_move(board: &mut Board, input: &str) -> u64 {
    let parse_move_result = parse_move(input);
    let mut try_move_result = Err(());
    match parse_move_result {
        Ok(potential_move) => try_move_result = try_move(board, potential_move),
        Err(e) => println!("Parsing error: {}", ERR_MV_STRINGS[e as usize]),
    }
    match try_move_result {
        Ok(()) => println!("Move played."),
        Err(()) if parse_move_result.is_ok() => println!("Illegal move."),
        Err(_) => (),
    }
    CMD_CONTINUE
}

fn parse_move(input: &str) -> ParseMoveResult {
    let length = input.len();
    let mut from: u8 = 0;
    let mut to: u8 = 0;
    let mut promotion_piece: Piece = PNONE;
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
        result = Ok((from, to, promotion_piece as u8));
    }
    result
}

// This function can be used to try and play the move resulting from parse_move().
fn try_move(board: &mut Board, potential_move: PotentialMove) -> Result<(), ()> {
    let mut move_list = MoveList::new();
    let mut result = Err(());

    board.gen_all_moves(&mut move_list);
    for i in 0..move_list.len() {
        let move_from_list = move_list.get_move(i);
        if_chain! {
            if potential_move.0 == move_from_list.from();
            if potential_move.1 == move_from_list.to();
            if potential_move.2 == move_from_list.promoted();
            if playmove::make(board, move_from_list);
            then {
                result = Ok(());
                break;
            }
        }
    }
    result
}
