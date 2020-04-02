use crate::defs::{BISHOP, ROOK};
use crate::extra::wizardry::find_magics;
use crate::utils::strip_newline;
use std::io::stdin;

pub fn get_input() -> u64 {
    let mut input = String::new();

    match stdin().read_line(&mut input) {
        Ok(_) => parse_input(&mut input),
        Err(error) => panic!("Error receiving input: {}", error),
    }
}

fn parse_input(input: &mut String) -> u64 {
    strip_newline(input);

    match &input[..] {
        "magics" => {
            find_magics(ROOK);
            find_magics(BISHOP);
            1
        }
        "test" => {
            println!("Testing...");
            1
        }
        "quit" => 0,
        _ => 1,
    }
}
