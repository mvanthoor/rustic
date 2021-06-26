/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

// This file implements Piece Square Tables (PST) for each piece type. The
// PST's are written from White's point of view, as if looking at a chess
// diagram, with A1 on the lower left corner.

use super::{
    defs::{PHASE_MAX, PHASE_MIN},
    Evaluation,
};
use crate::{
    board::Board,
    defs::{NrOf, Sides},
    misc::bits,
};

type Pst = [i16; NrOf::SQUARES];
type PstCollection = [Pst; NrOf::PIECE_TYPES];

// === MG Piece-Square Tables ===

#[rustfmt::skip]
const KING_MG: Pst =
    [  //KING MG
      -11,   70,   55,   31,  -37,  -16,   22,   22,
       37,   24,   25,   36,   16,    8,  -12,  -31,
       33,   26,   42,   11,   11,   40,   35,   -2,
        0,   -9,    1,  -21,  -20,  -22,  -15,  -60,
      -25,   16,  -27,  -67,  -81,  -58,  -40,  -62,
        7,   -2,  -37,  -77,  -79,  -60,  -23,  -26,
       12,   15,  -13,  -72,  -56,  -28,   15,   17,
       -6,   44,   29,  -58,    8,  -25,   34,   28,
    ];

#[rustfmt::skip]
const QUEEN_MG: Pst =
    [  //QUEEN MG
        865,  902,  922,  911,  964,  948,  933,  928,
        886,  865,  903,  921,  888,  951,  923,  940,
        902,  901,  907,  919,  936,  978,  965,  966,
        881,  885,  897,  894,  898,  929,  906,  915,
        907,  884,  899,  896,  904,  906,  912,  911,
        895,  916,  900,  902,  904,  912,  924,  917,
        874,  899,  918,  908,  915,  924,  911,  906,
        906,  899,  906,  918,  898,  890,  878,  858,
    ];

#[rustfmt::skip]
const ROOK_MG: Pst =
    [  //ROOK MG
        493,  511,  487,  515,  514,  483,  485,  495,
        493,  498,  529,  534,  546,  544,  483,  508,
        465,  490,  499,  497,  483,  519,  531,  480,
        448,  464,  476,  495,  484,  506,  467,  455,
        442,  451,  468,  470,  476,  472,  498,  454,
        441,  461,  468,  465,  478,  481,  478,  452,
        443,  472,  467,  476,  483,  500,  487,  423,
        459,  463,  470,  479,  480,  480,  446,  458,
    ];

#[rustfmt::skip]
const BISHOP_MG: Pst =
    [  //BISHOP MG
        292,  338,  254,  283,  299,  294,  337,  323,
        316,  342,  319,  319,  360,  385,  343,  295,
        342,  377,  373,  374,  368,  392,  385,  363,
        332,  338,  356,  384,  370,  380,  337,  341,
        327,  354,  353,  366,  373,  346,  345,  341,
        335,  350,  351,  347,  352,  361,  350,  344,
        333,  354,  354,  339,  344,  353,  367,  333,
        309,  341,  342,  325,  334,  332,  302,  313,
    ];

#[rustfmt::skip]
const KNIGHT_MG: Pst =    
    [  //KNIGHT MG
        116,  228,  271,  270,  338,  213,  278,  191,
        225,  247,  353,  331,  321,  360,  300,  281,
        258,  354,  343,  362,  389,  428,  375,  347,
        300,  332,  325,  360,  349,  379,  339,  333,
        298,  322,  325,  321,  337,  332,  332,  303,
        287,  297,  316,  319,  327,  320,  327,  294,
        276,  259,  300,  304,  308,  322,  296,  292,
        208,  290,  257,  274,  296,  284,  293,  284,
    ];

#[rustfmt::skip]
const PAWN_MG: Pst =
    [  //PAWN MG
        100,  100,  100,  100,  100,  100,  100,  100,
        176,  214,  147,  194,  189,  214,  132,   77,
        82,   88,  106,  113,  150,  146,  110,   73,
        67,   93,   83,   95,   97,   92,   99,   63,
        55,   74,   80,   89,   94,   86,   90,   55,
        55,   70,   68,   69,   76,   81,  101,   66,
        52,   84,   66,   60,   69,   99,  117,   60,
        100,  100,  100,  100,  100,  100,  100,  100,
    ];

// === EG Piece-Square Tables ===

#[rustfmt::skip]
const KING_EG: Pst =
    [  //KING EG
      -74,  -43,  -23,  -25,  -11,   10,    1,  -12,
      -18,    6,    4,    9,    7,   26,   14,    8,
       -3,    6,   10,    6,    8,   24,   27,    3,
      -16,    8,   13,   20,   14,   19,   10,   -3,
      -25,  -14,   13,   20,   24,   15,    1,  -15,
      -27,  -10,    9,   20,   23,   14,    2,  -12,
      -32,  -17,    4,   14,   15,    5,  -10,  -22,
      -55,  -40,  -23,   -6,  -20,   -8,  -28,  -47,
    ];

#[rustfmt::skip]
const QUEEN_EG: Pst =
    [  //QUEEN EG
      918,  937,  943,  945,  934,  926,  924,  942,
      907,  945,  946,  951,  982,  933,  928,  912,
      896,  921,  926,  967,  963,  937,  924,  915,
      926,  944,  939,  962,  983,  957,  981,  950,
      893,  949,  942,  970,  952,  956,  953,  936,
      911,  892,  933,  928,  934,  942,  934,  924,
      907,  898,  883,  903,  903,  893,  886,  888,
      886,  887,  890,  872,  916,  890,  906,  879,
    ];

#[rustfmt::skip]
const ROOK_EG: Pst =
    [  //ROOK EG
      506,  500,  508,  502,  504,  507,  505,  503,
      505,  506,  502,  502,  491,  497,  506,  501,
      504,  503,  499,  500,  500,  495,  496,  496,
      503,  502,  510,  500,  502,  504,  500,  505,
      505,  509,  509,  506,  504,  503,  496,  495,
      500,  503,  500,  505,  498,  498,  499,  489,
      496,  495,  502,  505,  498,  498,  491,  499,
      492,  497,  498,  496,  493,  493,  497,  480,
    ];

#[rustfmt::skip]
const BISHOP_EG: Pst =
    [  //BISHOP EG
      288,  278,  287,  292,  293,  290,  287,  277,
      289,  294,  301,  288,  296,  289,  294,  281,
      292,  289,  296,  292,  296,  300,  296,  293,
      293,  302,  305,  305,  306,  302,  296,  297,
      289,  293,  304,  308,  298,  301,  291,  288,
      285,  294,  304,  303,  306,  294,  290,  280,
      285,  284,  291,  299,  300,  290,  284,  271,
      277,  292,  286,  295,  294,  288,  290,  285,
    ];

#[rustfmt::skip]
const KNIGHT_EG: Pst =
    [  //KNIGHT EG
      229,  236,  269,  250,  257,  249,  219,  188,
      252,  274,  263,  281,  273,  258,  260,  229,
      253,  264,  290,  289,  278,  275,  263,  243,
      267,  280,  299,  301,  299,  293,  285,  264,
      263,  273,  293,  301,  296,  293,  284,  261,
      258,  276,  278,  290,  287,  274,  260,  255,
      241,  259,  270,  277,  276,  262,  260,  237,
      253,  233,  258,  264,  261,  260,  234,  215,
    ];

#[rustfmt::skip]
const PAWN_EG: Pst =
    [  //PAWN EG
      100,  100,  100,  100,  100,  100,  100,  100,
      277,  270,  252,  229,  240,  233,  264,  285,
      190,  197,  182,  168,  155,  150,  180,  181,
      128,  117,  108,  102,   93,  100,  110,  110,
      107,  101,   89,   85,   86,   83,   92,   91,
       96,   96,   85,   92,   88,   83,   85,   82,
      107,   99,   97,   97,  100,   89,   89,   84,
      100,  100,  100,  100,  100,  100,  100,  100,
    ];

pub const PST_MG: PstCollection = [KING_MG, QUEEN_MG, ROOK_MG, BISHOP_MG, KNIGHT_MG, PAWN_MG];
pub const PST_EG: PstCollection = [KING_EG, QUEEN_EG, ROOK_EG, BISHOP_EG, KNIGHT_EG, PAWN_EG];

// To make the Piece Square tables easier to relate to, and easier to
// edit, they have been laid out as a normal chess board, with A1 at
// the lower left. Because the square numbers start with A1 = 0, a
// conversion needs to be done. Below are the representations of the
// square numbers and the PST from both white and black point of view.

// (These tables will hold piece/square values instead of coordinates.
// The coordinates are used to visualize to which square a value in the
// table would belong.)

// Square numbers, as they are in arrays:

//  0  1  2  3  4  5  6  7   <= 0 = A1, 7 = H1
//  8  9 10 11 12 13 14 15
// 16 17 18 19 20 21 22 23
// 24 25 26 27 28 29 30 31
// 32 33 34 35 36 37 38 39
// 40 41 42 43 44 45 46 47
// 48 49 50 51 52 53 54 55
// 56 57 58 59 60 61 62 63  <= 56 = A8, 63 = H8

// PST, WHITE:                // Same PST, BLACK:

// A8 B8 C8 D8 E8 F8 G8 H8  |  // A1 B1 C1 D1 E1 G1 F1 H1
// A7 B7 C7 D8 E8 F8 G7 H7  |  // A2 B2 C2 D2 E2 G2 F2 H2
// A6 B6 C6 D6 E6 F6 G6 H6  |  // A3 B3 C3 D3 E3 G3 F3 H3
// A5 B5 C5 D5 E5 F5 G5 H5  |  // A4 B4 C4 D4 E4 G4 F4 H4
// A4 B4 C4 D4 E4 F4 G4 H4  |  // A5 B5 C5 D5 E5 G5 F5 H5
// A3 B3 C3 D3 E3 F3 G3 H3  |  // A6 B6 C6 D6 E6 F6 G6 H6
// A2 B2 C2 D2 E2 F2 G2 H2  |  // A7 B7 C7 D7 E7 G7 F7 H7
// A1 B1 C1 D1 E1 F1 G1 H1  |  // A8 B8 C8 D8 E8 F8 G8 H8

// If one super-imposes the square numbers on the PST with square names
// from WHITE's point of view, it can be seen that the following is true:

/*
    Square to PST element examples:
    A1 = square 0.  ==> PST element 56.
    H8 = square 63. ==> PST element 7.
    E8 = square 60. ==> PST element 4.
    E1 = square 4.  ==> PST element 60.
*/

// One can also see that, if the SAME PST from WHITE's point of view is to
// be used for BLACK, it can be indexed by the square number, without any
// converstion. (Super-impose the square numbers on top of the PST with
// BLACK square names.)

// This results in the following converstion table, from aquare number
// to PST element, needed for WHITE only::

#[allow(dead_code)]
#[rustfmt::skip]
pub const FLIP: [usize; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63,
    48, 49, 50, 51, 52, 53, 54, 55,
    40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39,
    24, 25, 26, 27, 28, 29, 30, 31,
    16, 17, 18, 19, 20, 21, 22, 23,
     8,  9, 10, 11, 12, 13, 14, 15,
     0,  1,  2,  3,  4,  5,  6,  7,
];

impl Evaluation {
    // Apply the PST's to the current position. These are the initial
    // values. The engine will update them incrementally during play.
    pub fn pst_apply(board: &Board, pst_collection: &PstCollection) -> (i16, i16) {
        let mut pst_w: i16 = 0;
        let mut pst_b: i16 = 0;
        let bb_white = board.bb_pieces[Sides::WHITE]; // Array of white piece bitboards
        let bb_black = board.bb_pieces[Sides::BLACK]; // Array of black piece bitboards

        // Iterate through the white and black bitboards (at the same time.)
        for (piece_type, (w, b)) in bb_white.iter().zip(bb_black.iter()).enumerate() {
            let mut white_pieces = *w; // White pieces of type "piece_type"
            let mut black_pieces = *b; // Black pieces of type "piece_type"

            // Iterate over pieces of the current piece_type for white.
            while white_pieces > 0 {
                let square = bits::next(&mut white_pieces);
                pst_w += pst_collection[piece_type][FLIP[square]] as i16;
            }

            // Iterate over pieces of the current piece_type for black.
            while black_pieces > 0 {
                let square = bits::next(&mut black_pieces);
                pst_b += pst_collection[piece_type][square] as i16;
            }
        }

        (pst_w, pst_b)
    }

    // Interpolate PST values between midgame and endgame tables. This
    // makes the engine much stronger, because it can now take into account
    // that piece values and locations are different in the opening/midgame
    // and endgame.
    pub fn pst_score(board: &Board) -> i16 {
        // Get current PST values. These are kept incrementally during play.
        let pst_w_mg = board.game_state.pst_mg[Sides::WHITE] as f32;
        let pst_b_mg = board.game_state.pst_mg[Sides::BLACK] as f32;
        let pst_w_eg = board.game_state.pst_eg[Sides::WHITE] as f32;
        let pst_b_eg = board.game_state.pst_eg[Sides::BLACK] as f32;

        // Get the game phase, from 1 (opening/midgame) to 0 (endgame)
        let v = board.game_state.phase_value;
        let phase = Evaluation::determine_phase(PHASE_MIN, PHASE_MAX, v);

        // Mix the tables by taking parts of both mg and eg.
        let score_w = (pst_w_mg * phase) + (pst_w_eg * (1.0 - phase));
        let score_b = (pst_b_mg * phase) + (pst_b_eg * (1.0 - phase));

        // Return final PST score.
        (score_w - score_b).round() as i16
    }
}
