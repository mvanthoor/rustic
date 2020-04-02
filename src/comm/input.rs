use crate::utils::parse::strip_newline;
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
        "quit" | "exit" => 0,
        _ => parse_move(input),
    }
}

fn parse_move(input: &mut String) -> u64 {
    if (4..=5).contains(&input.len()) {
        println!("ok");
    }
    1
}
