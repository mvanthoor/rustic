use librustic::evaluation::defs::{FlipTable, PhaseValues, FLIP};
use librustic::evaluation::defs::{Psqt, PsqtSet, W};

#[rustfmt::skip]
const PSQT_KING: Psqt = 
[
    W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0),
    W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0),
    W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0),
    W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0),
    W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0),
    W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0),
    W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0),
    W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0), W(0,0),
];

#[rustfmt::skip]
const PSQT_QUEEN: Psqt = 
[
    W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900),
    W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900),
    W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900),
    W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900),
    W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900),
    W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900),
    W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900),
    W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900), W(900,900),
];

#[rustfmt::skip]
const PSQT_ROOK: Psqt = 
[
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
    W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500), W(500,500),
];

#[rustfmt::skip]
const PSQT_BISHOP: Psqt = 
[
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
];

#[rustfmt::skip]
const PSQT_KNIGHT: Psqt =     
[
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
    W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300), W(300,300),
];

#[rustfmt::skip]
const PSQT_PAWN: Psqt = 
[
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
    W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100), W(100,100),
];

const PSQT_SET: PsqtSet = [
    PSQT_KING,
    PSQT_QUEEN,
    PSQT_ROOK,
    PSQT_BISHOP,
    PSQT_KNIGHT,
    PSQT_PAWN,
];

const PHASE_VALUES: PhaseValues = [0, 900, 500, 300, 300, 0];
const PHASE_MIN: i16 = 300;
const PHASE_MAX: i16 = 900;

struct EvalParams;
impl EvalParams {
    pub const FLIP: FlipTable = FLIP;
    pub const PSQT_SET: PsqtSet = PSQT_SET;
    pub const PHASE_VALUES: PhaseValues = self::PHASE_VALUES;
    pub const PHASE_MIN: i16 = self::PHASE_MIN;
    pub const PHASE_MAX: i16 = self::PHASE_MAX;
}
