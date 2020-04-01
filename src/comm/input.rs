use crate::utils::strip_newline;
use std::io::stdin;

pub fn get_move() -> u64 {
    let mut input = String::new();

    match stdin().read_line(&mut input) {
        Ok(_) => parse_move(&mut input),
        Err(error) => panic!("Error receiving input: {}", error),
    }
}

pub fn parse_move(input: &mut String) -> u64 {
    strip_newline(input);

    let length = &input.len();
    if (4..=5).contains(length) {
        println!("All right, next one...");
        1
    } else {
        0
    }
}
