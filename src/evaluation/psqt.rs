// This file implements Piece Square Tables (PST) for each piece type. The
// PST's are written from White's point of view, as if looking at a chess
// diagram, with A1 on the lower left corner.

use crate::{
    board::Board,
    defs::{NrOf, Sides},
    evaluation::defs::{EvalParams, W},
    evaluation::Evaluation,
    misc::bits,
};

type Psqt = [W; NrOf::SQUARES];
pub type FlipTable = [usize; NrOf::SQUARES];
pub type PsqtSet = [Psqt; NrOf::PIECE_TYPES];

#[rustfmt::skip]
const PSQT_KING: Psqt = 
[
    W(0,-95), W(0,-95), W( 0,-90), W(  0,-90), W(  0,-90), W(0,-90), W( 0,-95), W(0,-95),
    W(0,-95), W(0,-50), W( 0,-50), W(  0,-50), W(  0,-50), W(0,-50), W( 0,-50), W(0,-95),
    W(0,-90), W(0,-50), W( 0,-20), W(  0,-20), W(  0,-20), W(0,-20), W( 0,-50), W(0,-90),
    W(0,-90), W(0,-50), W( 0,-20), W( 20,  0), W( 20,  0), W(0,-20), W( 0,-50), W(0,-90),
    W(0,-90), W(0,-50), W( 0,-20), W( 20,  0), W( 20,  0), W(0,-20), W( 0,-50), W(0,-90),
    W(0,-90), W(0,-50), W( 0,-20), W(  0,-20), W(  0,-20), W(0,-20), W( 0,-50), W(0,-90),
    W(0,-95), W(0,-50), W( 0,-50), W(-10,-50), W(-10,-50), W(0,-50), W( 0,-50), W(0,-95),
    W(0,-95), W(0,-95), W(20,-90), W(-10,-90), W(-10,-90), W(0,-90), W(20,-95), W(0,-95),
];

#[rustfmt::skip]
const PSQT_QUEEN: Psqt = 
[
    W(865,918), W(902,937), W(922,943), W(911,945), W(964,934), W(948,926), W(933,924), W(928,942),
    W(886,907), W(865,945), W(903,946), W(921,951), W(888,982), W(951,933), W(923,928), W(940,912),
    W(902,896), W(901,921), W(907,926), W(919,967), W(936,963), W(978,937), W(965,924), W(966,915),
    W(881,926), W(885,944), W(897,939), W(894,962), W(898,983), W(929,957), W(906,981), W(915,950),
    W(907,893), W(884,949), W(899,942), W(896,970), W(904,952), W(906,956), W(912,953), W(911,936),
    W(895,911), W(916,892), W(900,933), W(902,928), W(904,934), W(912,942), W(924,934), W(917,924),
    W(874,907), W(899,898), W(918,883), W(908,903), W(915,903), W(924,893), W(911,886), W(906,888),
    W(906,886), W(899,887), W(906,890), W(918,872), W(898,916), W(890,890), W(878,906), W(858,879),
];

#[rustfmt::skip]
const PSQT_ROOK: Psqt = 
[
    W(493,506), W(511,500), W(487,508), W(515,502), W(514,504), W(483,507), W(485,505), W(495,503),
    W(493,505), W(498,506), W(529,502), W(534,502), W(546,491), W(544,497), W(483,506), W(508,501),
    W(465,504), W(490,503), W(499,499), W(497,500), W(483,500), W(519,495), W(531,496), W(480,496),
    W(448,503), W(464,502), W(476,510), W(495,500), W(484,502), W(506,504), W(467,500), W(455,505),
    W(442,505), W(451,509), W(468,509), W(470,506), W(476,504), W(472,503), W(498,496), W(454,495),
    W(441,500), W(461,503), W(468,500), W(465,505), W(478,498), W(481,498), W(478,499), W(452,489),
    W(443,496), W(472,495), W(467,502), W(476,505), W(483,498), W(500,498), W(487,491), W(423,499),
    W(459,492), W(463,497), W(470,498), W(479,496), W(480,493), W(480,493), W(446,497), W(458,480),
];

#[rustfmt::skip]
const PSQT_BISHOP: Psqt = 
[
    W(292,288), W(338,278), W(254,287), W(283,292), W(299,293), W(294,290), W(337,287), W(323,277),
    W(316,289), W(342,294), W(319,301), W(319,288), W(360,296), W(385,289), W(343,294), W(295,281),
    W(342,292), W(377,289), W(373,296), W(374,292), W(368,296), W(392,300), W(385,296), W(363,293),
    W(332,293), W(338,302), W(356,305), W(384,305), W(370,306), W(380,302), W(337,296), W(341,297),
    W(327,289), W(354,293), W(353,304), W(366,308), W(373,298), W(346,301), W(345,291), W(341,288),
    W(335,285), W(350,294), W(351,304), W(347,303), W(352,306), W(361,294), W(350,290), W(344,280),
    W(333,285), W(354,284), W(354,291), W(339,299), W(344,300), W(353,290), W(367,284), W(333,271),
    W(309,277), W(341,292), W(342,286), W(325,295), W(334,294), W(332,288), W(302,290), W(313,285),
];

#[rustfmt::skip]
const PSQT_KNIGHT: Psqt =     
[
    W(116,229), W(228,236), W(271,269), W(270,250), W(338,257), W(213,249), W(278,219), W(191,188),
    W(225,252), W(247,274), W(353,263), W(331,281), W(321,273), W(360,258), W(300,260), W(281,229),
    W(258,253), W(354,264), W(343,290), W(362,289), W(389,278), W(428,275), W(375,263), W(347,243),
    W(300,267), W(332,280), W(325,299), W(360,301), W(349,299), W(379,293), W(339,285), W(333,264),
    W(298,263), W(322,273), W(325,293), W(321,301), W(337,296), W(332,293), W(332,284), W(303,261),
    W(287,258), W(297,276), W(316,278), W(319,290), W(327,287), W(320,274), W(327,260), W(294,255),
    W(276,241), W(259,259), W(300,270), W(304,277), W(308,276), W(322,262), W(296,260), W(292,237),
    W(208,253), W(290,233), W(257,258), W(274,264), W(296,261), W(284,260), W(293,234), W(284,215),
];

#[rustfmt::skip]
const PSQT_PAWN: Psqt = 
[
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
    W(176,277), W(214,270), W(147,252), W(194,229), W(189,240), W(214,233), W(132,264), W(77, 285),
    W(82, 190), W(88, 197), W(106,182), W(113,168), W(150,155), W(146,150), W(110,180), W(73, 181),
    W(67, 128), W(93, 117), W(83, 108), W(95, 102), W(97,  93), W(92, 100), W(99, 110), W(63, 110),
    W(55, 107), W(74, 101), W(80,  89), W(89,  85), W(94,  86), W(86,  83), W(90,  92), W(55,  91),
    W(55,  96), W(70,  96), W(68,  85), W(69,  92), W(76,  88), W(81,  83), W(101, 85), W(66,  82),
    W(52, 107), W(84,  99), W(66,  97), W(60,  97), W(69, 100), W(99,  89), W(117, 89), W(60,  84),
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
];

pub const PSQT_SET: PsqtSet = [
    PSQT_KING,
    PSQT_QUEEN,
    PSQT_ROOK,
    PSQT_BISHOP,
    PSQT_KNIGHT,
    PSQT_PAWN,
];

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
// conversation. (Super-impose the square numbers on top of the PST with
// BLACK square names.)

// This results in the following conversation table, from Square number
// to PST element, needed for WHITE only::

#[allow(dead_code)]
#[rustfmt::skip]
pub const FLIP: FlipTable = [
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
    // Apply the PSQT's to the current position. This will only be done
    // during board initialization, after a position has been loaded by the
    // FEN-reader. The outcome will be placed in the board's game state, so
    // the engine can update the values incrementally very time a piece is
    // moved.
    pub fn psqt_apply(board: &Board, psqt_set: &PsqtSet) -> (W, W) {
        let mut psqt_w_mg: i16 = 0;
        let mut psqt_w_eg: i16 = 0;
        let mut psqt_b_mg: i16 = 0;
        let mut psqt_b_eg: i16 = 0;
        let bb_white = board.bb_pieces[Sides::WHITE]; // Array of white piece bitboards
        let bb_black = board.bb_pieces[Sides::BLACK]; // Array of black piece bitboards

        // Iterate through the white and black bitboards (at the same time.)
        for (piece_type, (w, b)) in bb_white.iter().zip(bb_black.iter()).enumerate() {
            let mut white_pieces = *w; // White pieces of type "piece_type"
            let mut black_pieces = *b; // Black pieces of type "piece_type"

            // Iterate over pieces of the current piece_type for white.
            while white_pieces > 0 {
                let square = bits::next(&mut white_pieces);
                psqt_w_mg += psqt_set[piece_type][FLIP[square]].mg();
                psqt_w_eg += psqt_set[piece_type][FLIP[square]].eg();
            }

            // Iterate over pieces of the current piece_type for black.
            while black_pieces > 0 {
                let square = bits::next(&mut black_pieces);
                psqt_b_mg += psqt_set[piece_type][square].mg();
                psqt_b_eg += psqt_set[piece_type][square].eg()
            }
        }

        (W(psqt_w_mg, psqt_w_eg), W(psqt_b_mg, psqt_b_eg))
    }

    // Interpolate PST values between midgame and endgame tables. This
    // makes the engine much stronger, because it can now take into account
    // that piece values and locations are different in the opening/midgame
    // and endgame.
    pub fn psqt_score(board: &Board) -> i16 {
        // Get current PST values. These are kept incrementally during play.
        let psqt_w_mg = board.game_state.psqt_value[Sides::WHITE].mg() as f32;
        let psqt_b_mg = board.game_state.psqt_value[Sides::BLACK].mg() as f32;
        let psqt_w_eg = board.game_state.psqt_value[Sides::WHITE].eg() as f32;
        let psqt_b_eg = board.game_state.psqt_value[Sides::BLACK].eg() as f32;

        // Get the game phase, from 1 (opening/midgame) to 0 (endgame)
        let v = board.game_state.phase_value;
        let phase = Evaluation::determine_phase(EvalParams::PHASE_MIN, EvalParams::PHASE_MAX, v);

        // Mix the tables by taking parts of both mg and eg.
        let score_w = (psqt_w_mg * phase) + (psqt_w_eg * (1.0 - phase));
        let score_b = (psqt_b_mg * phase) + (psqt_b_eg * (1.0 - phase));

        // Return final PST score.
        (score_w - score_b).round() as i16
    }
}
