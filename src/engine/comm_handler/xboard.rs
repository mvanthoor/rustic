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

use crate::{
    comm::{CommOutput, XBoardInput},
    engine::{defs::ErrFatal, Engine},
    evaluation::Evaluation,
};

impl Engine {
    pub fn xboard_handler(&mut self, command: &XBoardInput) {
        match command {
            XBoardInput::Quit => self.quit(),
            XBoardInput::Ping(value) => self.comm.send(CommOutput::Pong(*value)),

            // Custom commands
            XBoardInput::Board => self.comm.send(CommOutput::PrintBoard),

            XBoardInput::History => self.comm.send(CommOutput::PrintHistory),

            XBoardInput::Eval => {
                let mtx_board = &self.board.lock().expect(ErrFatal::LOCK);
                let eval = Evaluation::evaluate_position(mtx_board);
                let p_v = mtx_board.game_state.phase_value;
                let msg = format!("Evaluation: {} centipawns, phase value: {}", eval, p_v);
                self.comm.send(CommOutput::InfoString(msg));
            }

            XBoardInput::Help => self.comm.send(CommOutput::PrintHelp),
            XBoardInput::Unknown => (),
        }
    }
}
