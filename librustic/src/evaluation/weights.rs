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
