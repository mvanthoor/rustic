use crate::defs::ENGINE;
use crate::utils::parse::strip_newline;
use std::io::{stdin, stdout, Write};

const CMD_ERROR: &str = "Command-line i/o error";

pub fn get_input() -> u64 {
    let mut input = String::new();

    print!("{} > ", ENGINE);
    match stdout().flush() {
        Ok(()) => (),
        Err(error) => panic!("{}: {}", CMD_ERROR, error),
    }
    match stdin().read_line(&mut input) {
        Ok(_) => parse_input(&mut input),
        Err(error) => panic!("{}: {}", CMD_ERROR, error),
    }
}

fn parse_input(input: &mut String) -> u64 {
    strip_newline(input);

    match &input[..] {
        "quit" | "exit" => 0,
        "clear" => cmd_clear(),
        _ => cmd_parse_move(input),
    }
}

fn cmd_clear() -> u64 {
    1
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
    1
}
