use crate::defs::NrOf;
use crate::evaluation::defs::EvalParams;
use crate::{board::Board, defs::Sides, evaluation::Evaluation, misc::bits};

pub type PhaseValues = [i16; NrOf::PIECE_TYPES];
pub const PHASE_VALUES: PhaseValues = [0, 1050, 405, 305, 155, 0];
pub const PHASE_MIN: i16 = 435;
pub const PHASE_MAX: i16 = 5255;

impl Evaluation {
    // Counts all the phase values for white and black and returns the
    // total result. This is the initial game phase. The engine will update
    // it incrementally during play as pieces are traded.
    pub fn count_phase(board: &Board) -> i16 {
        let mut phase_w: i16 = 0;
        let mut phase_b: i16 = 0;
        let bb_w = board.bb_pieces[Sides::WHITE];
        let bb_b = board.bb_pieces[Sides::BLACK];

        for (piece_type, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            let mut white_pieces = *w;
            let mut black_pieces = *b;

            while white_pieces > 0 {
                phase_w += EvalParams::PHASE_VALUES[piece_type];
                bits::next(&mut white_pieces);
            }

            while black_pieces > 0 {
                phase_b += EvalParams::PHASE_VALUES[piece_type];
                bits::next(&mut black_pieces);
            }
        }

        phase_w + phase_b
    }

    // Get the game phase by using the Linstep method.
    pub fn determine_phase(edge0: i16, edge1: i16, value: i16) -> f32 {
        // Interpolate from edge0 to edge1.
        let result = (value - edge0) as f32 / (edge1 - edge0) as f32;

        // Clamp the result: don't drop below 0.0 or exceed 1.0.
        result.clamp(0.0, 1.0)
    }
}
