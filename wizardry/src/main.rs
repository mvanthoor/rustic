mod wizardry;

use librustic::board::defs::Pieces;

fn main() {
    wizardry::find_magics(Pieces::ROOK);
    wizardry::find_magics(Pieces::BISHOP);
}
