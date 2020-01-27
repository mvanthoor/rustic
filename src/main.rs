mod board;
mod defines;
mod fen;
mod move_boards;
mod print;

use board::Board;
use defines::*;
use move_boards::MoveBoard;

fn main() {
    let mut board: Board = Default::default();
    let mut move_board: MoveBoard = Default::default();

    println!();
    println!("{} {}, by {}", ENGINE, VERSION, AUTHOR);

    board.initialize();
    move_board.initialize();
    print::position(&board);
    print::bitboard(move_board.tmp_rook[0]);

    // Test generation of all blockers, iterative
    pub type SuperBit = [u8; 8];
    let mut super_bit: SuperBit = [0; 8];
    let mut super_bit_list: Vec<SuperBit> = Vec::new();

    println!();
    for i in 0..super_bit.len() {
        if i == 0 {
            super_bit_list.push(super_bit);
            super_bit[i] = 1;
            super_bit_list.push(super_bit);
            super_bit[i] = 0;
        } else {
            for j in 0..super_bit_list.len() {
                let mut sb = super_bit_list[j];
                sb[i] = 1;
                super_bit_list.push(sb);
            }
        }
    }

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
