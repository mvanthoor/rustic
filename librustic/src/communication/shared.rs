use crate::search::defs::{CHECKMATE, CHECKMATE_THRESHOLD};

pub fn moves_to_checkmate(score: i16) -> Option<i16> {
    let detected = (score.abs() >= CHECKMATE_THRESHOLD) && (score.abs() < CHECKMATE);
    if detected {
        let plies = CHECKMATE - score.abs();
        let is_odd = plies % 2 == 1;
        let moves = if is_odd { (plies + 1) / 2 } else { plies / 2 };
        Some(moves)
    } else {
        None
    }
}
