pub const ASCII_VALUE_OF_LOWERCASE_A: u8 = 97;
pub const ASCII_VALUE_OF_1: u8 = 49;
pub const COORDINATE_LETTERS: &str = "abcdefgh";
pub const COORDINATE_NUMBERS: &str = "12345678";

const ERR_STR_SQUARE_DOESNT_EXIST: &str = "This square doesn't exist.";

pub fn strip_newline(input: &mut String) {
    for _ in 0..2 {
        let c = input.chars().next_back();
        let cr = if let Some('\r') = c { true } else { false };
        let lf = if let Some('\n') = c { true } else { false };

        if cr || lf {
            input.pop();
        }
    }
}

pub fn algebraic_square_to_number(algebraic_move: &str) -> Result<u8, &str> {
    let length = algebraic_move.len();
    let mut result: Result<u8, &str> = Err(ERR_STR_SQUARE_DOESNT_EXIST);

    if length == 2 {
        let mut file = 0;
        let mut rank = 0;
        let mut char_ok = 0;

        for (index, c) in algebraic_move.to_lowercase().chars().enumerate() {
            if index == 0 && COORDINATE_LETTERS.contains(c) {
                file = (c as u8) - ASCII_VALUE_OF_LOWERCASE_A;
                char_ok += 1;
            }
            if index == 1 && COORDINATE_NUMBERS.contains(c) {
                rank = (c as u8) - ASCII_VALUE_OF_1;
                char_ok += 1;
            }
        }

        if char_ok == length {
            let square_nr = (rank * 8) + file;
            result = Ok(square_nr);
        }
    }
    result
}
