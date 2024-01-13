use crate::evaluation::psqt::{FlipTable, FLIP};
use crate::evaluation::psqt::{PsqtSet, PSQT_SET};
pub use crate::evaluation::weights::W;

pub struct EvalParams;
impl EvalParams {
    pub const FLIP: FlipTable = FLIP;
    pub const PSQT_SET: PsqtSet = PSQT_SET;
    pub const PHASE_VALUES: [i16; 6] = [0, 1050, 405, 305, 155, 0];
    pub const PHASE_MIN: i16 = 435;
    pub const PHASE_MAX: i16 = 5255;
}
