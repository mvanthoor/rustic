use crate::board::representation::Board;
use crate::defs::ENGINE;
use crate::extra::perft;
use crate::extra::perftsuite;
use crate::utils::parse::strip_newline;
use std::io::{stdin, stdout, Write};

const CMD_STR_ERROR: &str = "Command-line i/o error";
const CMD_QUIT: u64 = 0;
const CMD_CONTINUE: u64 = 1;

pub fn get_input(board: &mut Board) -> u64 {
    let mut input = String::new();

    print!("{} > ", ENGINE);
    match stdout().flush() {
        Ok(()) => (),
        Err(error) => panic!("{}: {}", CMD_STR_ERROR, error),
    }
    match stdin().read_line(&mut input) {
        Ok(_) => parse_input(&mut input, board),
        Err(error) => panic!("{}: {}", CMD_STR_ERROR, error),
    }
}

fn parse_input(input: &mut String, board: &mut Board) -> u64 {
    strip_newline(input);

    match &input[..] {
        "quit" | "exit" => CMD_QUIT,
        "perft" => cmd_perft(board),
        "suite" => cmd_suite(board),
        "clear" => cmd_clear(),
        _ => cmd_parse_move(input),
    }
}

fn cmd_clear() -> u64 {
    CMD_CONTINUE
}

fn cmd_perft(board: &Board) -> u64 {
    perft::run(board, 7);
    CMD_CONTINUE
}

fn cmd_suite(board: &Board) -> u64 {
    perftsuite::run_all_tests();
    CMD_CONTINUE
}

fn cmd_parse_move(input: &mut String) -> u64 {
    let length = input.len();
    if (4..=5).contains(&length) {
        for (index, c) in input.chars().enumerate() {
            match index {
                0 | 2 => println!("file: {}", c),
                1 | 3 => println!("rank: {}", c),
                4 => println!("Promotion! {}", c),
                _ => (),
            }
        }
    }
    CMD_CONTINUE
}
