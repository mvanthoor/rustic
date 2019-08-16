use crate::defs::*;
use crate::Board;

const MAILBOX_FILES: u8 = 10;
const MAILBOX_RANKS: u8 = 12;
const MAILBOX_SIZE: usize = (MAILBOX_FILES * MAILBOX_RANKS) as usize;
const INVALID_FILES: [u8; 2] = [0, 9];
const INVALID_RANKS: [u8; 4] = [0, 1, 10, 11];
const INVALID_SQUARE: u8 = 255;

pub struct HelperBoard {
    pub mailbox: [u8; MAILBOX_SIZE],
    pub real: [u8; 64],
}

/*
The Default function creates these helper boards.
They are temporary, to help generate the bitboard
move boards.

mailbox to real board:

255 255 255 255 255 255 255 255 255 255     (119)
255 255 255 255 255 255 255 255 255 255     (109)
255  56  57  58  59  60  61  62  63 255     (99)
255  48  49  50  51  52  53  54  55 255     (89)
255  40  41  42  43  44  45  46  47 255     (79)
255  32  33  34  35  36  37  38  39 255     (69)
255  24  25  26  27  28  29  30  31 255     (59)
255  16  17  18  19  20  21  22  23 255     (49)
255   8   9  10  11  12  13  14  15 255     (39)
255   0   1   2   3   4   5   6   7 255     (29)
255 255 255 255 255 255 255 255 255 255     (19)
255 255 255 255 255 255 255 255 255 255     (9)

real to mailbox board:

8)  91 92 93 94 95 96 97 98     (63)
7)  81 82 83 84 85 86 87 88     (55)
6)  71 72 73 74 75 76 77 78     (47)
5)  61 62 63 64 65 66 67 68     (39)
4)  51 52 53 54 55 56 57 58     (31)
3)  41 42 43 44 45 46 47 48     (23)
2)  31 32 33 34 35 36 37 38     (15)
1)  21 22 23 24 25 26 27 28     (7)

    A  B  C  D  E  F  G  H
*/

impl Default for HelperBoard {
    fn default() -> HelperBoard {
        let mut helper_board: HelperBoard = HelperBoard {
            mailbox: [0; MAILBOX_SIZE],
            real: [0; 64],
        };
        let mut real_board_square: usize = 0;

        for rank in 0..MAILBOX_RANKS {
            for file in 0..MAILBOX_FILES {
                let square = ((rank * MAILBOX_FILES) + file) as usize;
                if INVALID_FILES.contains(&file) || INVALID_RANKS.contains(&rank) {
                    helper_board.mailbox[square] = INVALID_SQUARE;
                } else {
                    helper_board.mailbox[square] = real_board_square as u8;
                    helper_board.real[real_board_square] = square as u8;
                    real_board_square += 1;
                }
            }
        }

        helper_board
    }
}

fn non_slider(board: &mut Board, mask: usize, directions: [i8; 8], h: &HelperBoard) {
    let nr_of_masks = board.bb_mask[mask].len();
    for i in 0..nr_of_masks {
        for &d in directions.iter() {
            let mailbox_square = h.real[i] as i8;
            let try_square = (mailbox_square + d) as usize;
            if h.mailbox[try_square] != INVALID_SQUARE {
                let legal_square = h.mailbox[try_square];
                board.bb_mask[mask][i] |= 1 << legal_square;
            }
        }
    }
}

fn slider(board: &mut Board, mask: usize, directions: [i8; 4], h: &HelperBoard) {
    let nr_of_masks = board.bb_mask[mask].len();
    for i in 0..nr_of_masks {
        for &d in directions.iter() {
            let mut current_mailbox_square = h.real[i] as i8;
            let mut next_mailbox_square = current_mailbox_square + d;
            while h.mailbox[next_mailbox_square as usize] != INVALID_SQUARE {
                current_mailbox_square += d;
                next_mailbox_square += d;
                if h.mailbox[next_mailbox_square as usize] != INVALID_SQUARE {
                    let add_square = h.mailbox[current_mailbox_square as usize];
                    board.bb_mask[mask][i] |= 1 << add_square;
                }
            }
        }
    }
}

pub fn create(board: &mut Board) {
    let helper_board: HelperBoard = Default::default();
    let directions_king: [i8; 8] = [-11, -10, -9, -1, 1, 9, 10, 11];
    let directions_knight: [i8; 8] = [-21, -19, -12, -8, 8, 12, 19, 21];
    let directions_rook: [i8; 4] = [-10, -1, 1, 10];
    let directions_bishop: [i8; 4] = [-11, -9, 9, 11];
    non_slider(board, BB_MASK_K, directions_king, &helper_board);
    non_slider(board, BB_MASK_N, directions_knight, &helper_board);
    slider(board, BB_MASK_R, directions_rook, &helper_board);
    slider(board, BB_MASK_B, directions_bishop, &helper_board);
}
