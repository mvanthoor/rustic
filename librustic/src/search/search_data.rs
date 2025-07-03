use crate::{
    movegen::defs::ShortMove, search::defs::CHECKMATE_THRESHOLD, transposition::defs::HashData,
};

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub enum HashFlag {
    #[default]
    Nothing,
    Exact,
    Alpha,
    Beta,
}

#[derive(Copy, Clone, Default)]
pub struct SearchData {
    depth: i8,
    flag: HashFlag,
    value: i16,
    best_move: ShortMove,
}

impl HashData for SearchData {
    fn empty() -> Self {
        Self::default()
    }

    fn depth(&self) -> i8 {
        self.depth
    }
}

impl SearchData {
    pub fn new(depth: i8, ply: i8, flag: HashFlag, value: i16, best_move: ShortMove) -> Self {
        // This is the value we're going to save into the TT.
        let mut v = value;

        // If the score we are handling is a checkmate score, we need to do
        // a little extra work. This is because we store checkmates in the
        // table using their distance from the node they're found in, not
        // their distance from the root. So if we found a checkmate-in-8 in
        // a node that was 5 plies from the root, we need to store the
        // score as a checkmate-in-3. Then, if we read the checkmate-in-3
        // from the table in a node that's 4 plies from the root, we need
        // to return the score as checkmate-in-7. (Comment taken from the
        // engine Blunder, by Christian Dean. It explained this better than
        // my comment did.)

        // We do not use a match, statement with comparison or guards,
        // because two if-statements are faster. In the TT, this speed
        // difference is significant.

        if v > CHECKMATE_THRESHOLD {
            v += ply as i16;
        }

        if v < -CHECKMATE_THRESHOLD {
            v -= ply as i16;
        }

        Self {
            depth,
            flag,
            value: v,
            best_move,
        }
    }

    pub fn get(&self, depth: i8, ply: i8, alpha: i16, beta: i16) -> (Option<i16>, ShortMove) {
        // We either do, or don't have a value to return from the TT.
        let mut value: Option<i16> = None;

        if self.depth >= depth {
            match self.flag {
                HashFlag::Exact => {
                    // Get the value from the data. We don't want to change
                    // the value that is in the TT.
                    let mut v = self.value;

                    // Opposite of storing a mate score in the TT...
                    if v > CHECKMATE_THRESHOLD {
                        v -= ply as i16;
                    }

                    if v < -CHECKMATE_THRESHOLD {
                        v += ply as i16;
                    }

                    // This is the value that will be returned.
                    value = Some(v);
                }

                HashFlag::Alpha => {
                    if self.value <= alpha {
                        value = Some(alpha);
                    }
                }

                HashFlag::Beta => {
                    if self.value >= beta {
                        value = Some(beta);
                    }
                }

                HashFlag::Nothing => (),
            }
        }
        (value, self.best_move)
    }
}
