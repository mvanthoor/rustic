pub use crate::evaluation::phase::PhaseValues;
use crate::evaluation::phase::{self};
use crate::evaluation::psqt::PSQT_SET;
pub use crate::evaluation::psqt::{FlipTable, PsqtSet, FLIP};
pub use crate::evaluation::weights::W;

#[cfg(feature = "extra")]
pub use crate::evaluation::psqt::Psqt;

pub struct EvalParams;
impl EvalParams {
    pub const FLIP: FlipTable = FLIP;
    pub const PSQT_SET: PsqtSet = PSQT_SET;
    pub const PHASE_VALUES: PhaseValues = phase::PHASE_VALUES;
    pub const PHASE_MIN: i16 = phase::PHASE_MIN;
    pub const PHASE_MAX: i16 = phase::PHASE_MAX;
}
