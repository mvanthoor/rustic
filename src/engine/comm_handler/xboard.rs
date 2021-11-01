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
            // Send a NewLine to the GUI to flush the output buffer.
            XBoardIn::XBoard => self.comm.send(CommOut::XBoard(XBoardOut::NewLine)),

            // Send the engine's supported features to the GUI.
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

            // Set up a new game from the starting position.
            XBoardIn::New => {
                // Abaondon the search if it is running.
                if self.is_analyzing() || self.is_thinking() {
                    self.search.send(SearchControl::Abandon);
                }

                // Set up a new game and then wait.
                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_read(Some(FEN_START_POSITION))
                    .expect(ErrFatal::NEW_GAME);
                self.tt_search.lock().expect(ErrFatal::LOCK).clear();
                self.set_waiting();
            }

            // Return to observation state. First abandon the search if the
            // engine is thinking.
            XBoardIn::Force => {
                if self.is_thinking() {
                    self.search.send(SearchControl::Abandon);
                    self.set_observing();
                } else if self.is_waiting() {
                    self.set_observing();
                } else {
                    self.comm.send(CommOut::Error(
                        ErrNormal::COMMAND_INVALID.to_string(),
                        command.to_string(),
                    ));
                }
            }

            // Set up the board according the incoming FEN-string.
            XBoardIn::SetBoard(fen) => {
                let fen_result = self.board.lock().expect(ErrFatal::LOCK).fen_read(Some(fen));
                if fen_result.is_err() {
                    self.comm.send(CommOut::Error(
                        ErrNormal::INCORRECT_FEN.to_string(),
                        fen.to_string(),
                    ));
                }
            }

            // Accept an incoming usermove. Execute it on the board. Then
            // react to this usermove, according to the state we were in
            // when the move was received.
            XBoardIn::UserMove(m) => {
                if self.is_observing() || self.is_waiting() {
                    if self.execute_move(m.clone()) {
                        self.send_game_result();
                    } else {
                        let illegal_move = CommOut::XBoard(XBoardOut::IllegalMove(m.clone()));
                        self.comm.send(illegal_move);
                    }
                } else {
                    self.comm.send(CommOut::Error(
                        ErrNormal::COMMAND_INVALID.to_string(),
                        command.to_string(),
                    ));
                }
            }

            // Send "Pong" to confirm that the engine is still active.
            XBoardIn::Ping(value) => self.comm.send(CommOut::XBoard(XBoardOut::Pong(*value))),

            // Enable sending search information.
            XBoardIn::Post => self.settings.verbosity = Verbosity::Full,

            // Disable sending search information.
            XBoardIn::NoPost => self.settings.verbosity = Verbosity::Silent,

            // Set the transposition table size.
            XBoardIn::Memory(mb) => self.tt_search.lock().expect(ErrFatal::LOCK).resize(*mb),

            // Start analyze mode. First abandon the search if the engine
            // is thinking.
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

            // Send intermediate statistics to the GUI.
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

            // Stop analyzing and return to "observe" state.
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

            // Some incoming commands are buffered by the comm module. The
            // engine is notified that a command was received and buffered.
            // The engine will output a message to confirm that the command
            // was received and handled.
            XBoardIn::Buffered(cmd) => self.comm.send(CommOut::Message(format!(
                "{}: {}",
                Messages::INCOMING_CMD_BUFFERED.to_string(),
                cmd.to_string()
            ))),
        }
    }
}
