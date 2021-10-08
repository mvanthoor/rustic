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
    comm::{CommOutput, XBoardInput, XBoardOutput},
    defs::FEN_START_POSITION,
    engine::{
        defs::{EngineStatus, ErrFatal},
        Engine,
    },
    search::defs::{SearchControl, SearchMode, SearchParams},
};

impl Engine {
    pub fn xboard_handler(&mut self, command: &XBoardInput) {
        const PROTOCOL_VERSION: u8 = 2;
        let mut sp = SearchParams::new();

        match command {
            XBoardInput::XBoard => self.comm.send(CommOutput::XBoard(XBoardOutput::NewLine)),

            XBoardInput::ProtoVer(n) => {
                if *n == PROTOCOL_VERSION {
                    self.comm
                        .send(CommOutput::XBoard(XBoardOutput::SendFeatures));
                }
            }

            XBoardInput::New => {
                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_read(Some(FEN_START_POSITION))
                    .expect(ErrFatal::NEW_GAME);
                self.tt_search.lock().expect(ErrFatal::LOCK).clear();
            }

            XBoardInput::SetBoard(fen) => {
                let fen_result = self.board.lock().expect(ErrFatal::LOCK).fen_read(Some(fen));
                if fen_result.is_err() {
                    let msg = String::from("Error: Incorrect FEN-string.");
                    self.comm.send(CommOutput::Message(msg));
                }
            }

            XBoardInput::UserMove(m) => {
                let ok = self.execute_move(m.clone());
                if !ok {
                    let illegal_move = CommOutput::XBoard(XBoardOutput::IllegalMove(m.clone()));
                    self.comm.send(illegal_move);
                }
            }

            XBoardInput::Go => {
                // Temporarily hardcoded.
                sp.depth = 12;
                sp.search_mode = SearchMode::Depth;
                self.search.send(SearchControl::Start(sp));
            }

            // Stops searching and provides the best move so far. This
            // command is ignored when not searching.
            XBoardInput::MoveNow => {
                if self.status == EngineStatus::Searching {
                    self.search.send(SearchControl::Stop)
                }
            }

            XBoardInput::Ping(value) => self
                .comm
                .send(CommOutput::XBoard(XBoardOutput::Pong(*value))),

            XBoardInput::Memory(mb) => self.tt_search.lock().expect(ErrFatal::LOCK).resize(*mb),

            XBoardInput::Analyze => {
                sp.search_mode = SearchMode::Infinite;
                self.search.send(SearchControl::Start(sp));
            }

            // Exits the search/analyze mode without producing a move. This
            // command is ignored when not analyzing.
            XBoardInput::Exit => {
                if self.status == EngineStatus::Analyzing {
                    self.search.send(SearchControl::Exit)
                }
            }
        }
    }
}
