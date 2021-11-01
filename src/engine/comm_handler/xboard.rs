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
    comm::{CommOut, XBoardIn, XBoardOut},
    defs::FEN_START_POSITION,
    engine::{
        defs::{ErrFatal, ErrNormal, Messages, Verbosity},
        Engine,
    },
    search::defs::{SearchControl, SearchMode, SearchParams},
};

impl Engine {
    pub fn xboard_handler(&mut self, command: &XBoardIn) {
        const PROTOCOL_VERSION: u8 = 2;

        let mut sp = SearchParams::new();
        sp.verbosity = self.settings.verbosity;

        match command {
            XBoardIn::XBoard => self.comm.send(CommOut::XBoard(XBoardOut::NewLine)),

            XBoardIn::ProtoVer(n) => {
                if self.is_observing() {
                    if *n == PROTOCOL_VERSION {
                        self.comm.send(CommOut::XBoard(XBoardOut::Features));
                    }
                } else {
                    self.comm.send(CommOut::Error(
                        ErrNormal::COMMAND_INVALID.to_string(),
                        command.to_string(),
                    ));
                }
            }

            XBoardIn::New => {
                if self.is_analyzing() || self.is_thinking() {
                    self.search.send(SearchControl::Abandon);
                }

                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_read(Some(FEN_START_POSITION))
                    .expect(ErrFatal::NEW_GAME);
                self.tt_search.lock().expect(ErrFatal::LOCK).clear();
                self.set_waiting();
            }

            XBoardIn::Force => {
                if self.is_analyzing() || self.is_thinking() {
                    self.search.send(SearchControl::Abandon);
                }

                self.set_observing();
            }

            XBoardIn::SetBoard(fen) => {
                let fen_result = self.board.lock().expect(ErrFatal::LOCK).fen_read(Some(fen));
                if fen_result.is_err() {
                    self.comm.send(CommOut::Error(
                        ErrNormal::INCORRECT_FEN.to_string(),
                        fen.to_string(),
                    ));
                }
            }

            XBoardIn::UserMove(m) => {
                // Execute the incoming user move...
                if self.execute_move(m.clone()) {
                    self.send_game_result();
                } else {
                    let illegal_move = CommOut::XBoard(XBoardOut::IllegalMove(m.clone()));
                    self.comm.send(illegal_move);
                }
            }

            XBoardIn::Ping(value) => self.comm.send(CommOut::XBoard(XBoardOut::Pong(*value))),

            XBoardIn::Post => self.settings.verbosity = Verbosity::Full,

            XBoardIn::NoPost => self.settings.verbosity = Verbosity::Silent,

            XBoardIn::Memory(mb) => self.tt_search.lock().expect(ErrFatal::LOCK).resize(*mb),

            XBoardIn::Analyze => {
                if !self.is_analyzing() {
                    if self.is_thinking() {
                        self.search.send(SearchControl::Abandon);
                    }

                    sp.search_mode = SearchMode::Infinite;
                    self.search.send(SearchControl::Start(sp));
                    self.set_analyzing();
                }
            }

            XBoardIn::Dot => {
                if self.is_analyzing() {
                    self.comm.send(CommOut::XBoard(XBoardOut::Stat01));
                } else {
                    self.comm.send(CommOut::Error(
                        ErrNormal::COMMAND_INVALID.to_string(),
                        command.to_string(),
                    ));
                }
            }

            XBoardIn::Exit => {
                if self.is_analyzing() {
                    // We order the search / analysis to stop and abandon
                    // any result such as the best move.
                    self.search.send(SearchControl::Abandon);
                    self.set_observing();
                } else {
                    self.comm.send(CommOut::Error(
                        ErrNormal::COMMAND_INVALID.to_string(),
                        command.to_string(),
                    ));
                }
            }

            XBoardIn::Buffered(cmd) => self.comm.send(CommOut::Message(format!(
                "{}: {}",
                Messages::INCOMING_CMD_BUFFERED.to_string(),
                cmd.to_string()
            ))),
        }
    }
}
