use super::{Attacks, Blockers, EMPTY, MAX_PERMUTATIONS};
use crate::defines::Bitboard;
use crate::utils::{create_bb_ray, Direction};

pub fn create_blocker_boards(mask: Bitboard) -> Blockers {
    let d: Bitboard = mask;
    let mut bb_blocker_boards: Blockers = [EMPTY; MAX_PERMUTATIONS];
    let mut n: Bitboard = 0;
    let mut i = 0;

    loop {
        bb_blocker_boards[i] = n;
        n = n.wrapping_sub(d) & d;
        i += 1;
        if n == 0 {
            break;
        }
    }

    bb_blocker_boards
}

pub fn create_rook_attack_boards(blockers: Blockers, square: u8) -> Attacks {
    let mut bb_attack_boards: Attacks = [EMPTY; MAX_PERMUTATIONS];
    for (i, bb_blocker) in blockers.iter().enumerate() {
        let bb_attacks = create_bb_ray(*bb_blocker, square, Direction::Up)
            | create_bb_ray(*bb_blocker, square, Direction::Right)
            | create_bb_ray(*bb_blocker, square, Direction::Down)
            | create_bb_ray(*bb_blocker, square, Direction::Left);
        bb_attack_boards[i] = bb_attacks;
    }

    bb_attack_boards
}
