use super::rays;
use crate::board::Direction;
use crate::defs::{Bitboard, Square};

pub type AttackBoards = Vec<Bitboard>;

// This function takes a square, and all the blocker boards belonging to that
// squre. Then it'll iterate through those blocker boards, and generate the
// attack board belonging to that blocker board. The 'length' parameter is the
// length of the given array of blocker boards.
pub fn create_rook_attack_boards(square: Square, blockers: &[Bitboard]) -> AttackBoards {
    let mut bb_attack_boards: AttackBoards = Vec::new();

    for b in blockers.iter() {
        let bb_attacks = rays::create_bb_ray(*b, square, Direction::Up)
            | rays::create_bb_ray(*b, square, Direction::Right)
            | rays::create_bb_ray(*b, square, Direction::Down)
            | rays::create_bb_ray(*b, square, Direction::Left);
        bb_attack_boards.push(bb_attacks);
    }

    bb_attack_boards
}

// Same as the function above, but for the bishop.
pub fn create_bishop_attack_boards(square: Square, blockers: &[Bitboard]) -> AttackBoards {
    let mut bb_attack_boards: AttackBoards = Vec::new();

    for b in blockers.iter() {
        let bb_attacks = rays::create_bb_ray(*b, square, Direction::UpLeft)
            | rays::create_bb_ray(*b, square, Direction::UpRight)
            | rays::create_bb_ray(*b, square, Direction::DownRight)
            | rays::create_bb_ray(*b, square, Direction::DownLeft);
        bb_attack_boards.push(bb_attacks);
    }

    bb_attack_boards
}
