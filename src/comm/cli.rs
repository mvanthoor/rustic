use crate::board::representation::Board;
use crate::defs::ENGINE;
use crate::extra::{perft, perftsuite};
use crate::movegen::movedefs::MoveList;
use crate::utils::parse;
use if_chain::if_chain;
use std::{io, io::Write};

const CMD_STR_ERR_IO: &str = "Command-line i/o error";
const CMD_STR_ERR_MOVE: &str = "Error in move input.";
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
    let mut error = 0;

    if (4..=5).contains(&length) {
        if_chain! {
            if let Ok(f) = parse::algebraic_square_to_number(&input[0..=1]);
            if let Ok(t) = parse::algebraic_square_to_number(&input[2..=3]);
            then {
                from = f;
                to = t;
            } else {
                error = 1;
            }
        }
    };
    if error == 0 {
        let mut move_list = MoveList::new();
        board.gen_all_moves(&mut move_list);

        println!("from: {}, to: {}", from, to);
    } else {
        println!("{}", CMD_STR_ERR_MOVE);
    }
    CMD_CONTINUE
}
