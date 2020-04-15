// TODO: Update comments

use crate::board::representation::Board;
use crate::defs::{ENGINE, PNONE};
use crate::extra::{perft, perftsuite};
use crate::utils::parse;
use if_chain::if_chain;
use std::{io, io::Write};

const CMD_STR_ERR_IO: &str = "Command-line i/o error";
const CMD_QUIT: u64 = 0;
const CMD_CONTINUE: u64 = 1;

pub fn get_input(board: &mut Board) -> u64 {
    let mut input = String::new();

    print!("{} > ", ENGINE);
    match io::stdout().flush() {
        Ok(()) => (),
        Err(error) => panic!("{}: {}", CMD_STR_ERR_IO, error),
    }
    match io::stdin().read_line(&mut input) {
        Ok(_) => parse_input(&mut input, board),
        Err(error) => panic!("{}: {}", CMD_STR_ERR_IO, error),
    }
}

fn parse_input(input: &mut String, board: &mut Board) -> u64 {
    parse::strip_newline(input);
    match &input[..] {
        "quit" | "exit" => CMD_QUIT,
        "perft" => cmd_perft(board),
        "suite" => cmd_suite(),
        "clear" => cmd_clear(),
        _ => cmd_parse_move(input, board),
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

fn cmd_parse_move(input: &mut String, board: &Board) -> u64 {
    let length = input.len();
    let mut from: u8 = 0;
    let mut to: u8 = 0;
    let mut promotion_piece = PNONE;
    let mut result = 0;

    if length == 4 || length == 5 {
        if_chain! {
            if let Ok(f) = parse::algebraic_square_to_number(&input[0..=1]);
            if let Ok(t) = parse::algebraic_square_to_number(&input[2..=3]);
            then {
                from = f;
                to = t;
            } else {
                result = 1;
            }
        };
    }

    if length == 5 {
        if_chain! {
            if let Some(c) = input.chars().next_back();
            if let Ok(p) = parse::piece_letter_to_number(c);
            then {
                promotion_piece = p;
            } else {
                result = 2;
            }
        }
    }

    if (length != 4) && (length != 5) {
        result = 3;
    }

    match result {
        0 => println!("From: {}, To: {}, Promotion: {}", from, to, promotion_piece),
        1 => println!("Error in square notation."),
        2 => println!("Not a promotion piece."),
        3 => println!("A move is 4 or 5 characters long (e2e4, a7a8q)."),
        _ => {}
    };

    CMD_CONTINUE
}
