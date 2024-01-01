pub use crate::evaluation::{psqt::FLIP, psqt::PSQT_COLLECTION, Evaluation};
pub const PHASE_VALUES: [i16; 6] = [0, 1050, 405, 305, 155, 0];
pub const PHASE_MIN: i16 = 435;
pub const PHASE_MAX: i16 = 5255;

// PSQT Weight struct First value: middlegame. Second value: endgame.
#[derive(Clone, Copy)]
pub struct W(pub i16, pub i16);
impl W {
    pub fn mg(&self) -> i16 {
        self.0
    }

    pub fn eg(&self) -> i16 {
        self.1
    }

    pub fn add(&mut self, w: W) {
        self.0 += w.0;
        self.1 += w.1;
    }

    pub fn sub(&mut self, w: W) {
        self.0 -= w.0;
        self.1 -= w.1;
    }
}
