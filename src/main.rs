mod board;
mod defs;
mod fen;
mod masks;
mod print;

use board::Board;
use defs::*;

fn main() {
    let mut board: Board = Default::default();

    println!();
    println!("{} {}, by {}", ENGINE, VERSION, AUTHOR);

    board.create_start_position();
    print::position(&board);

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

    for i in 0..super_bit_list.len() {
        let sb = super_bit_list[i];
        print!("{}: ", i);
        for j in 0..sb.len() {
            print!("{}", sb[j]);
        }
        println!();
    }
}
