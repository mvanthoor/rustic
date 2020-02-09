use crate::defines::*;

const MAILBOX_FILES: u8 = 10;
const MAILBOX_RANKS: u8 = 12;
const MAILBOX_SIZE: usize = (MAILBOX_FILES * MAILBOX_RANKS) as usize;
const INVALID_FILES: [u8; 2] = [0, 9];
const INVALID_RANKS: [u8; 4] = [0, 1, 10, 11];
const INVALID_SQUARE: u8 = 255;

/*
The helper board is based on the 10x12 mailbox concept.
It is used to help generate the bitboard-based move boards;
the helper board itself is then discarded and not used
during play.

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

pub struct HelperBoard {
    pub mailbox: [u8; MAILBOX_SIZE],
    pub real: [u8; 64],
}

impl Default for HelperBoard {
    fn default() -> HelperBoard {
        let mut helper_board: HelperBoard = HelperBoard {
            mailbox: [0; MAILBOX_SIZE],
            real: [0; NR_OF_SQUARES as usize],
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

pub struct Magics {
    pub king: NonSliderAttacks,
    pub knight: NonSliderAttacks,
    pub tmp_rook: SliderMoves,
    pub tmp_bishop: SliderMoves,
    pub tmp_queen: SliderMoves,
}

impl Default for Magics {
    fn default() -> Magics {
        Magics {
            king: [0; NR_OF_SQUARES as usize],
            knight: [0; NR_OF_SQUARES as usize],
            tmp_rook: [0; NR_OF_SQUARES as usize],
            tmp_bishop: [0; NR_OF_SQUARES as usize],
            tmp_queen: [0; NR_OF_SQUARES as usize],
        }
    }
}

impl Magics {
    fn non_slider(&mut self, piece: Piece, directions: NonSliderDirections, helper: &HelperBoard) {
        for sq in 0..NR_OF_SQUARES {
            for d in directions.iter() {
                let square = sq as usize;
                let mailbox_square = helper.real[square] as i8;
                let try_square = (mailbox_square + d) as usize;
                if helper.mailbox[try_square] != INVALID_SQUARE {
                    let valid_square = helper.mailbox[try_square];
                    match piece {
                        KING => self.king[square] |= 1 << valid_square,
                        KNIGHT => self.knight[square] |= 1 << valid_square,
                        _ => (),
                    }
                }
            }
        }
    }

    fn slider(&mut self, piece: Piece, directions: SliderDirections, helper: &HelperBoard) {
        for sq in 0..NR_OF_SQUARES {
            for d in directions.iter() {
                let square = sq as usize;
                let mut current_mailbox_square = helper.real[square] as i8;
                let mut next_mailbox_square = current_mailbox_square + d;
                while helper.mailbox[next_mailbox_square as usize] != INVALID_SQUARE {
                    current_mailbox_square += d;
                    next_mailbox_square += d;
                    if helper.mailbox[next_mailbox_square as usize] != INVALID_SQUARE {
                        let valid_square = helper.mailbox[current_mailbox_square as usize];
                        match piece {
                            ROOK => self.tmp_rook[square] |= 1 << valid_square,
                            BISHOP => self.tmp_bishop[square] |= 1 << valid_square,
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    fn slider_queen(&mut self) {
        for sq in 0..NR_OF_SQUARES {
            let square = sq as usize;
            self.tmp_queen[square] = self.tmp_bishop[square] ^ self.tmp_rook[square];
        }
    }

    pub fn initialize(&mut self) {
        let helper_board: HelperBoard = Default::default();
        let directions_king: NonSliderDirections = [-11, -10, -9, -1, 1, 9, 10, 11];
        let directions_knight: NonSliderDirections = [-21, -19, -12, -8, 8, 12, 19, 21];
        let directions_rook: SliderDirections = [-10, -1, 1, 10];
        let directions_bishop: SliderDirections = [-11, -9, 9, 11];

        self.non_slider(KING, directions_king, &helper_board);
        self.non_slider(KNIGHT, directions_knight, &helper_board);
        self.slider(ROOK, directions_rook, &helper_board);
        self.slider(BISHOP, directions_bishop, &helper_board);
        self.slider_queen();
    }
}

/*
    fn pawn(board: &mut Board, mask: usize, direction: i8, h: &HelperBoard) {
        let nr_of_masks = board.bb_mask[mask].len();
        let d = direction;
        for i in 8..nr_of_masks - 8 {
            let current_mailbox_square = h.real[i] as i8;
            let one_forward = h.mailbox[(current_mailbox_square + d) as usize];
            board.bb_mask[mask][i] |= 1 << one_forward;
            if current_mailbox_square >= 31 && current_mailbox_square <= 38 {
                let two_forward = h.mailbox[(current_mailbox_square + 2 * d) as usize];
                board.bb_mask[mask][i] |= 1 << two_forward;
            }
        }
    }
*/
