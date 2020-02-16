mod board;
mod defines;
mod fen;
mod magics;
mod movegen;
mod print;

use board::Board;
use defines::*;
use magics::Magics;
use movegen::*;

fn main() {
    let mut board: Board = Default::default();
    let mut magics: Magics = Default::default();
    let mut moves: MoveList = Vec::with_capacity(MAX_LEGAL_MOVES as usize);

    // let test_pos: &str = "3r4/2bk4/3PN3/7p/1pP2n2/3p3P/Nn1P1PP1/KR1B4 w - - 0 1";
    // let test_pos: &str = "4k3/ppp3pp/2Pp4/1P2p1p1/P3P2p/8/8/4K3 w - - 0 1";
    // let test_pos: &str = "4k3/8/8/2p1p1Pp/1p1p3P/p2PPP2/PPP2PPP/4K3 w - - 0 1";
    let test_pos: &str = "k7/8/8/8/2p5/5P2/1r1N4/KB3b2 w - - 0 1";
    board.initialize(test_pos);
    magics.initialize();

    print::engine_info();
    print::position(&board, None);

    movegen::generate(&board, WHITE, &magics, &mut moves);
    print::movelist(&moves);

    // for i in 0..64 {
    //     println!("{}", i);
    //     print::bitboard(magics.pawn_masks[0][i], Some(i as u8))
    // }

    println!("Done.");

    // Test generation of all blockers, iterative
    // pub type SuperBit = [u8; 8];
    // let mut super_bit: SuperBit = [0; 8];
    // let mut super_bit_list: Vec<SuperBit> = Vec::new();

    // println!();
    // for i in 0..super_bit.len() {
    //     if i == 0 {
    //         super_bit_list.push(super_bit);
    //         super_bit[i] = 1;
    //         super_bit_list.push(super_bit);
    //         super_bit[i] = 0;
    //     } else {
    //         for j in 0..super_bit_list.len() {
    //             let mut sb = super_bit_list[j];
    //             sb[i] = 1;
    //             super_bit_list.push(sb);
    //         }
    //     }
    // }

    /*
        for i in 0..super_bit_list.len() {
            let sb = super_bit_list[i];
            print!("{}: ", i);
            for j in 0..sb.len() {
                print!("{}", sb[j]);
            }
            println!();
        }
    */
}
