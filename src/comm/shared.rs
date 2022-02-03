/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2022, Marcel Vanthoor
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

use crate::{
    board::Board,
    engine::defs::{EngineState, ErrFatal},
    search::defs::{CHECKMATE, CHECKMATE_THRESHOLD},
};
use std::sync::{Arc, Mutex};

pub struct Shared {}

impl Shared {
    // This function returns the number of moves to mate, or None if no
    // mate is detected.
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

    pub fn print_board(board: &Arc<Mutex<Board>>) {
        println!("{}", &board.lock().expect(ErrFatal::LOCK));
    }

    pub fn print_history(board: &Arc<Mutex<Board>>) {
        let mtx_board = board.lock().expect(ErrFatal::LOCK);
        let length = mtx_board.history.len();

        if length > 0 {
            for i in 0..length {
                println!("{:<3}| ply: {} {}", i, i + 1, mtx_board.history.get_ref(i));
            }
        } else {
            println!("No history available.");
        }
    }

    pub fn print_eval(eval: i16, phase: i16) {
        println!("Evaluation: {}, Phase: {}", eval, phase);
    }

    pub fn print_state(state: &EngineState) {
        println!("State: {}", state);
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
        println!("state     :   Print current state of the engine.");
        println!("cleartt   :   Clear the transposition table.");
        println!();
    }
}
