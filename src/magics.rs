use crate::defines::{
    Bitboard, Piece, Side, ALL_SQUARES, BISHOP, BLACK, FILE_A, FILE_B, FILE_G, FILE_H, KING,
    KNIGHT, NR_OF_SQUARES, PAWN_SQUARES, RANK_1, RANK_2, RANK_7, RANK_8, ROOK, WHITE,
};
use crate::utils::{create_bb_files, create_bb_ranks};

const MAILBOX_FILES: u8 = 10;
const MAILBOX_RANKS: u8 = 12;
const MAILBOX_SIZE: usize = (MAILBOX_FILES * MAILBOX_RANKS) as usize;
const INVALID_FILES: [u8; 2] = [0, 9];
const INVALID_RANKS: [u8; 4] = [0, 1, 10, 11];
const INVALID_SQUARE: u8 = 255;
const WHITE_BLACK: usize = 2;
const NR_OF_RAYS: usize = 4;
const EMPTY: Bitboard = 0;
const NSQ: usize = NR_OF_SQUARES as usize;

// This is a total sum of all rook or bishop blocker permutations per square.
const ROOK_TABLE_SIZE: u32 = 102400;
const BISHOP_TABLE_SIZE: u32 = 5248;

type SliderDirections = [i8; 4];
type PawnCaptureDirections = [i8; 2];
type NonSliderAttacks = [Bitboard; NSQ];
type BlockerMask = [Bitboard; NSQ];
type SquaresInRays = [u8; NR_OF_RAYS];

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

struct HelperBoard {
    pub mailbox: [u8; MAILBOX_SIZE],
    pub real: [u8; NSQ],
}

impl Default for HelperBoard {
    fn default() -> HelperBoard {
        let mut helper_board: HelperBoard = HelperBoard {
            mailbox: [0; MAILBOX_SIZE],
            real: [0; NSQ],
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

struct SliderInfo {
    pub _rook_squares_in_ray: [SquaresInRays; NSQ],
    pub _rook_permutations_count: [u32; NSQ],
    pub _bishop_squares_in_ray: [SquaresInRays; NSQ],
    pub _bishop_permutations_count: [u32; NSQ],
}

// _rook_permutations_count: Rook blocker permutations per square
// _bishop_permutations_count: Bishop blocker permutations per square
// The total of each of these determines the size of the attack tables.
// Rook total permutations of all squares should be: 102400.
// Bishop total permutations of all squares should be: 5248.
impl Default for SliderInfo {
    fn default() -> SliderInfo {
        SliderInfo {
            _rook_squares_in_ray: [[0; NR_OF_RAYS]; NSQ],
            _rook_permutations_count: [0; NSQ],
            _bishop_squares_in_ray: [[0; NR_OF_RAYS]; NSQ],
            _bishop_permutations_count: [0; NSQ],
        }
    }
}

pub struct Magics {
    _king: NonSliderAttacks,
    _knight: NonSliderAttacks,
    _pawns: [NonSliderAttacks; WHITE_BLACK],
    _rook_blocker_mask: BlockerMask,
    _bishop_blocker_mask: BlockerMask,
    _rook_attacks: [Bitboard; ROOK_TABLE_SIZE as usize],
    _bishop_attacks: [Bitboard; BISHOP_TABLE_SIZE as usize],
}

impl Default for Magics {
    fn default() -> Magics {
        Magics {
            _king: [EMPTY; NSQ],
            _knight: [EMPTY; NSQ],
            _pawns: [[EMPTY; NSQ]; WHITE_BLACK],
            _rook_blocker_mask: [EMPTY; NSQ],
            _bishop_blocker_mask: [EMPTY; NSQ],
            _rook_attacks: [0; ROOK_TABLE_SIZE as usize],
            _bishop_attacks: [0; BISHOP_TABLE_SIZE as usize],
        }
    }
}

impl Magics {
    pub fn initialize(&mut self) {
        let helper_board: HelperBoard = Default::default();
        let files = create_bb_files();
        let ranks = create_bb_ranks();
        let mut slider_info: SliderInfo = Default::default();

        self.init_king(&files, &ranks);
        self.init_knight(&files, &ranks);
        self.init_pawn_attacks(&helper_board);
        self.create_blocker_mask(ROOK, &mut slider_info, &helper_board);
        self.create_blocker_mask(BISHOP, &mut slider_info, &helper_board);
        self.permutations_count(&mut slider_info);
        self.permutations_test(&slider_info);
    }

    pub fn get_non_slider_attacks(&self, piece: Piece, square: u8) -> Bitboard {
        let s = square as usize;

        match piece {
            KING => self._king[s],
            KNIGHT => self._knight[s],
            _ => 0,
        }
    }

    pub fn get_pawn_attacks(&self, side: Side, square: u8) -> Bitboard {
        debug_assert!(side == WHITE || side == BLACK, "Incorrect side.");
        self._pawns[side][square as usize]
    }

    fn init_king(&mut self, files: &[Bitboard; 8], ranks: &[Bitboard; 8]) {
        for sq in ALL_SQUARES {
            let square = 1u64 << sq;
            let moves = (square & !files[FILE_A] & !ranks[RANK_8]) << 7
                | (square & !ranks[RANK_8]) << 8
                | (square & !files[FILE_H] & !ranks[RANK_8]) << 9
                | (square & !files[FILE_H]) << 1
                | (square & !files[FILE_H] & !ranks[RANK_1]) >> 7
                | (square & !ranks[RANK_1]) >> 8
                | (square & !files[FILE_A] & !ranks[RANK_1]) >> 9
                | (square & !files[FILE_A]) >> 1;
            self._king[sq as usize] = moves;
        }
    }

    fn init_knight(&mut self, files: &[Bitboard; 8], ranks: &[Bitboard; 8]) {
        for sq in ALL_SQUARES {
            let square = 1u64 << sq;
            let moves = (square & !ranks[RANK_8] & !ranks[RANK_7] & !files[FILE_A]) << 15
                | (square & !ranks[RANK_8] & !ranks[RANK_7] & !files[FILE_H]) << 17
                | (square & !files[FILE_A] & !files[FILE_B] & !ranks[RANK_8]) << 6
                | (square & !files[FILE_G] & !files[FILE_H] & !ranks[RANK_8]) << 10
                | (square & !ranks[RANK_1] & !ranks[RANK_2] & !files[FILE_A]) >> 17
                | (square & !ranks[RANK_1] & !ranks[RANK_2] & !files[FILE_H]) >> 15
                | (square & !files[FILE_A] & !files[FILE_B] & !ranks[RANK_1]) >> 10
                | (square & !files[FILE_G] & !files[FILE_H] & !ranks[RANK_1]) >> 6;
            self._knight[sq as usize] = moves;
        }
    }

    fn init_pawn_attacks(&mut self, helper: &HelperBoard) {
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

    fn create_blocker_mask(&mut self, piece: Piece, info: &mut SliderInfo, helper: &HelperBoard) {
        debug_assert!(piece == ROOK || piece == BISHOP, "Incorrect piece.");
        const DIRECTIONS_ROOK: SliderDirections = [10, 1, -10, -1];
        const DIRECTIONS_BISHOP: SliderDirections = [11, -9, -11, 9];
        let directions = match piece {
            ROOK => DIRECTIONS_ROOK,
            BISHOP => DIRECTIONS_BISHOP,
            _ => [0; 4],
        };

        for sq in ALL_SQUARES {
            for (i, d) in directions.iter().enumerate() {
                let square = sq as usize;
                let mut current_mailbox_square = helper.real[square] as i8;
                let mut next_mailbox_square = current_mailbox_square + d;

                while helper.mailbox[next_mailbox_square as usize] != INVALID_SQUARE {
                    current_mailbox_square += d;
                    next_mailbox_square += d;
                    if helper.mailbox[next_mailbox_square as usize] != INVALID_SQUARE {
                        let valid_square = helper.mailbox[current_mailbox_square as usize];
                        match piece {
                            ROOK => {
                                self._rook_blocker_mask[square] |= 1u64 << valid_square;
                                info._rook_squares_in_ray[square][i] += 1;
                            }
                            BISHOP => {
                                self._bishop_blocker_mask[square] |= 1u64 << valid_square;
                                info._bishop_squares_in_ray[square][i] += 1;
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    fn permutations_count(&self, info: &mut SliderInfo) {
        for s in ALL_SQUARES {
            let mut rook_bits = 0;
            let mut bishop_bits = 0;
            let square = s as usize;
            for r in 0..NR_OF_RAYS {
                let ray = r as usize;
                rook_bits += info._rook_squares_in_ray[square][ray];
                bishop_bits += info._bishop_squares_in_ray[square][ray];
            }
            info._rook_permutations_count[square] = 2u32.pow(rook_bits as u32);
            info._bishop_permutations_count[square] = 2u32.pow(bishop_bits as u32);
        }
    }

    fn permutations_test(&mut self, info: &SliderInfo) {
        let count = info._rook_squares_in_ray[0][0];
        let permutations = 2u8.pow(count as u32);
        let mut store: Vec<Bitboard> = Vec::new();
        for i in 0..permutations {
            let current_permutation = i;
            let mut bitboard: Bitboard = 0;
            for j in 0..count {
                let bit_to_set_in_ray = (j + 1) * 8;
                let bit_from_permutation = (current_permutation >> j) & 1u8;
                bitboard |= (bit_from_permutation as u64) << (bit_to_set_in_ray as u64);
            }
            store.push(bitboard);
        }
    }
}
