mod board;
mod defines;
mod fen;
mod magics;
mod movegen;
mod print;

use board::Board;
use defines::*;
use magics::MoveBoard;

fn main() {
    let mut board: Board = Default::default();
    let mut move_board: MoveBoard = Default::default();

    let test_pos: &str = "1rk5/3R4/8/8/8/4r3/4PKb1/5RN1 w - - 0 1";
    board.initialize(test_pos);
    move_board.initialize();

    print::engine_info();
    print::position(&board, None);
    movegen::generate(&board, WHITE);

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
