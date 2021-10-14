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

mod uci;
mod xboard;

use crate::{
    comm::{CommInput, CommOutput},
    engine::{defs::ErrFatal, Engine},
    evaluation::Evaluation,
};

// This block implements handling of incoming information, which will be in
// the form of either Comm or Search reports.
impl Engine {
    pub fn comm_handler(&mut self, input: &CommInput) {
        match input {
            CommInput::Uci(command) => self.uci_handler(command),
            CommInput::XBoard(command) => self.xboard_handler(command),

            CommInput::Quit => self.quit(),
            CommInput::Board => self.comm.send(CommOutput::PrintBoard),
            CommInput::History => self.comm.send(CommOutput::PrintHistory),
            CommInput::Eval => {
                let mtx_board = &self.board.lock().expect(ErrFatal::LOCK);
                let eval = Evaluation::evaluate_position(mtx_board);
                let phase = mtx_board.game_state.phase_value;
                self.comm.send(CommOutput::PrintEval(eval, phase));
            }
            CommInput::Help => self.comm.send(CommOutput::PrintHelp),
            CommInput::Ok => (), // Input completely handled by comm module.
            CommInput::Unknown(cmd) => {
                let err_type = String::from("Unknown command");
                self.comm.send(CommOutput::Error((*cmd).clone(), err_type));
            }
        }
    }
}
