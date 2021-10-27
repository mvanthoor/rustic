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
    comm::{CommIn, CommOut},
    engine::{
        defs::{ErrFatal, Messages},
        Engine,
    },
    evaluation::Evaluation,
};

use super::defs::ErrNormal;

// This block implements handling of incoming information, which will be in
// the form of either Comm or Search reports.
impl Engine {
    pub fn comm_handler(&mut self, input: &CommIn) {
        match input {
            CommIn::Uci(command) => self.uci_handler(command),
            CommIn::XBoard(command) => self.xboard_handler(command),

            CommIn::Quit => self.quit(),
            CommIn::Board => self.comm.send(CommOut::PrintBoard),
            CommIn::History => self.comm.send(CommOut::PrintHistory),
            CommIn::Eval => {
                let mtx_board = &self.board.lock().expect(ErrFatal::LOCK);
                let eval = Evaluation::evaluate_position(mtx_board);
                let phase = mtx_board.game_state.phase_value;
                self.comm.send(CommOut::PrintEval(eval, phase));
            }
            CommIn::State => self.comm.send(CommOut::PrintState(self.state)),
            CommIn::Help => self.comm.send(CommOut::PrintHelp),
            CommIn::Ignore(cmd) => {
                self.comm.send(CommOut::Message(format!(
                    "{}: {}",
                    Messages::COMMAND_IGNORED.to_string(),
                    cmd
                )));
            }
            CommIn::Unknown(cmd) => self.comm.send(CommOut::Error(
                ErrNormal::UNKNOWN_COMMAND.to_string(),
                cmd.to_string(),
            )),
        }
    }
}
