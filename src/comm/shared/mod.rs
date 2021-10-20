/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

use crate::{board::Board, engine::defs::ErrFatal, misc::print};
use std::sync::{Arc, Mutex};

pub struct Shared {}

impl Shared {
    pub fn print_board(board: &Arc<Mutex<Board>>) {
        print::position(&board.lock().expect(ErrFatal::LOCK), None);
    }

    pub fn print_history(board: &Arc<Mutex<Board>>) {
        let mtx_board = board.lock().expect(ErrFatal::LOCK);
        let length = mtx_board.history.len();

        if length == 0 {
            println!("No history available.");
        }

        for i in 0..length {
            let h = mtx_board.history.get_ref(i);
            println!("{:<3}| ply: {} {}", i, i + 1, h.as_string());
        }

        std::mem::drop(mtx_board);
    }

    pub fn print_eval(eval: i16, phase: i16) {
        println!("Evaluation: {}, Phase: {}", eval, phase);
    }

    pub fn print_help(protocol: &str) {
        println!(
            "The engine is in {} communication mode. It supports some custom",
            protocol
        );
        println!(
            "non-{} commands to make use through a terminal window easier.",
            protocol
        );
        println!("These commands can also be very useful for debugging purposes.");
        println!();
        println!("Custom commands");
        println!("================================================================");
        println!("help      :   This help information.");
        println!("board     :   Print the current board state.");
        println!("history   :   Print a list of past board states.");
        println!("eval      :   Print evaluation for side to move.");
        println!();
    }
}
