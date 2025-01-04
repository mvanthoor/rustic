use librustic::{board::defs::Pieces, movegen::wizardry};

fn main() {
    wizardry::find_magics(Pieces::ROOK);
    wizardry::find_magics(Pieces::BISHOP);
}
