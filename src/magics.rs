use crate::defines::{
    Bitboard, Piece, Side, ALL_SQUARES, BISHOP, BLACK, KING, KNIGHT, NR_OF_SQUARES, PAWN_SQUARES,
    ROOK, WHITE,
};

const MAILBOX_FILES: u8 = 10;
const MAILBOX_RANKS: u8 = 12;
const MAILBOX_SIZE: usize = (MAILBOX_FILES * MAILBOX_RANKS) as usize;
const INVALID_FILES: [u8; 2] = [0, 9];
const INVALID_RANKS: [u8; 4] = [0, 1, 10, 11];
const INVALID_SQUARE: u8 = 255;
const WHITE_BLACK: usize = 2;

type SliderDirections = [i8; 4];
type NonSliderDirections = [i8; 8];
type PawnCaptureDirections = [i8; 2];
type NonSliderAttacks = [Bitboard; NR_OF_SQUARES as usize];
type MoveMask = [Bitboard; NR_OF_SQUARES as usize];

/*
The helper board is based on the 10x12 mailbox concept.
It is used to help generate the bitboard-based attack boards;
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
    _king: NonSliderAttacks,
    _knight: NonSliderAttacks,
    _pawns: [NonSliderAttacks; WHITE_BLACK],
    pub tmp_rook: MoveMask,
    pub tmp_bishop: MoveMask,
    pub tmp_queen: MoveMask,
}

impl Default for Magics {
    fn default() -> Magics {
        const EMPTY: Bitboard = 0;
        const NSQ: usize = NR_OF_SQUARES as usize;

        Magics {
            _king: [EMPTY; NSQ],
            _knight: [EMPTY; NSQ],
            _pawns: [[EMPTY; NSQ]; WHITE_BLACK],
            tmp_rook: [EMPTY; NSQ],
            tmp_bishop: [EMPTY; NSQ],
            tmp_queen: [EMPTY; NSQ],
        }
    }
}

impl Magics {
    pub fn initialize(&mut self) {
        let helper_board: HelperBoard = Default::default();

        self.non_slider(KING, &helper_board);
        self.non_slider(KNIGHT, &helper_board);
        self.pawns(&helper_board);
        self.slider(ROOK, &helper_board);
        self.slider(BISHOP, &helper_board);
        self.slider_queen();
    }

    pub fn get_non_slider_attacks(&self, piece: Piece, square: u8) -> Bitboard {
        let s = square as usize;

        match piece {
            KING => self._king[s],
            KNIGHT => self._king[s],
            _ => 0,
        }
    }

    pub fn get_pawn_attacks(&self, side: Side, square: u8) -> Bitboard {
        debug_assert!(side == WHITE || side == BLACK, "Incorrect side.");
        self._pawns[side][square as usize]
    }

    fn non_slider(&mut self, piece: Piece, helper: &HelperBoard) {
        debug_assert!(piece == KING || piece == KNIGHT, "Incorrect piece.");
        const DIRECTIONS_KING: NonSliderDirections = [-11, -10, -9, -1, 1, 9, 10, 11];
        const DIRECTIONS_KNIGHT: NonSliderDirections = [-21, -19, -12, -8, 8, 12, 19, 21];
        let directions = match piece {
            KING => DIRECTIONS_KING,
            KNIGHT => DIRECTIONS_KNIGHT,
            _ => [0; 8],
        };

        for sq in ALL_SQUARES {
            for d in directions.iter() {
                let square = sq as usize;
                let mailbox_square = helper.real[square] as i8;
                let try_square = (mailbox_square + d) as usize;

                if helper.mailbox[try_square] != INVALID_SQUARE {
                    let valid_square = helper.mailbox[try_square];
                    match piece {
                        KING => self._king[square] |= 1u64 << valid_square,
                        KNIGHT => self._knight[square] |= 1u64 << valid_square,
                        _ => (),
                    }
                }
            }
        }
    }

    fn slider(&mut self, piece: Piece, helper: &HelperBoard) {
        debug_assert!(piece == ROOK || piece == BISHOP, "Incorrect piece.");
        const DIRECTIONS_ROOK: SliderDirections = [-10, -1, 1, 10];
        const DIRECTIONS_BISHOP: SliderDirections = [-11, -9, 9, 11];
        let directions = match piece {
            ROOK => DIRECTIONS_ROOK,
            BISHOP => DIRECTIONS_BISHOP,
            _ => [0; 4],
        };

        for sq in ALL_SQUARES {
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
                            ROOK => self.tmp_rook[square] |= 1u64 << valid_square,
                            BISHOP => self.tmp_bishop[square] |= 1u64 << valid_square,
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    fn slider_queen(&mut self) {
        for sq in ALL_SQUARES {
            let square = sq as usize;
            self.tmp_queen[square] = self.tmp_bishop[square] ^ self.tmp_rook[square];
        }
    }

    fn pawns(&mut self, helper: &HelperBoard) {
        const DIRECTIONS: PawnCaptureDirections = [9, 11];

        for sq in PAWN_SQUARES {
            for d in DIRECTIONS.iter() {
                let square = sq as usize;
                let mailbox_square = helper.real[square] as i8;
                let try_square_white = (mailbox_square + d) as usize;
                let try_square_black = (mailbox_square + -d) as usize;

                if helper.mailbox[try_square_white] != INVALID_SQUARE {
                    let valid_square = helper.mailbox[try_square_white];
                    self._pawns[WHITE][square] |= 1u64 << valid_square;
                }
                if helper.mailbox[try_square_black] != INVALID_SQUARE {
                    let valid_square = helper.mailbox[try_square_black];
                    self._pawns[BLACK][square] |= 1u64 << valid_square;
                }
            }
        }
    }
}
