use super::{Attacks, EMPTY, MAX_PERMUTATIONS};
use crate::defines::Bitboard;
use crate::print;
use crate::utils::{create_bb_ray, Direction};

pub fn create_rook_attacks(mask: Bitboard, square: u8) -> Attacks {
    let d: Bitboard = mask;
    let mut bb_attack_boards: Attacks = [Some(EMPTY); MAX_PERMUTATIONS];
    let mut n: Bitboard = 0;
    let mut i = 0;

    loop {
        // Do something with subset n: in this case, create 4 rays
        let bb_r1 = create_bb_ray(n, square, Direction::Up);
        let bb_r2 = create_bb_ray(n, square, Direction::Right);
        let bb_r3 = create_bb_ray(n, square, Direction::Down);
        let bb_r4 = create_bb_ray(n, square, Direction::Left);
        let bb_attacks = bb_r1 | bb_r2 | bb_r3 | bb_r4;

        bb_attack_boards[i] = Some(bb_attacks);

        if i >= 0 && i <= 25 {
            println!("Run: {}", i);
            println!("Subset");
            print::bitboard(n, Some(square));
            println!("Attack board");
            print::bitboard(bb_attacks, Some(square));
        }

        n = n.wrapping_sub(d) & d;
        i += 1;
        if n == 0 {
            break;
        }
    }

    bb_attack_boards
}
