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
    comm::defs::{CommOut, TimeControl, XBoardIn, XBoardOut},
    defs::FEN_START_POSITION,
    engine::{
        defs::{EngineState, ErrFatal, ErrNormal, GameEndReason, Information, Messages, Verbosity},
        Engine,
    },
    search::defs::{SearchControl, SearchMode, SearchParams, SearchReport},
};

impl Engine {
    pub fn xboard_handler(&mut self, command: &XBoardIn) {
        const PROTOCOL_VERSION: u8 = 2;

        let mut search_params = SearchParams::new();
        search_params.verbosity = self.settings.verbosity;

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
                // Abandon the search if it is running.
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
                match self.state {
                    EngineState::Thinking => {
                        self.search.send(SearchControl::Abandon);
                        self.set_observing();
                    }
                    EngineState::Waiting => self.set_observing(),
                    _ => self.comm.send(CommOut::Error(
                        ErrNormal::COMMAND_INVALID.to_string(),
                        command.to_string(),
                    )),
                };
            }

            XBoardIn::Go(tc) => {
                if self.is_observing() || self.is_waiting() {
                    // Pass time control to search parameters and start
                    // thinking. We display an error if no time control has
                    // been set.
                    if tc.is_set() {
                        Engine::set_time_control(&mut search_params, tc);
                        self.search.send(SearchControl::Start(search_params));
                        self.set_thinking();
                    } else {
                        self.comm.send(CommOut::Error(
                            ErrNormal::TIME_CONTROL_NOT_SET.to_string(),
                            command.to_string(),
                        ));
                    }
                } else {
                    self.comm.send(CommOut::Error(
                        ErrNormal::COMMAND_INVALID.to_string(),
                        command.to_string(),
                    ));
                }
            }

            XBoardIn::QuestionMark => {
                if self.is_thinking() {
                    self.search.send(SearchControl::Stop);
                    self.set_waiting();
                } else {
                    self.comm.send(CommOut::Error(
                        ErrNormal::COMMAND_INVALID.to_string(),
                        command.to_string(),
                    ));
                }
            }

            // Set up the board according the incoming FEN-string.
            XBoardIn::SetBoard(fen) => {
                // If we were analyzing, we abandon the search, wait for
                // the search to become ready again, and clear the TT.
                if self.is_analyzing() {
                    self.search.send(SearchControl::Abandon);
                    while self.info_rx() != Information::Search(SearchReport::Ready) {}
                    self.tt_search.lock().expect(ErrFatal::LOCK).clear();
                }

                // Set up the new board.
                let fen_result = self.board.lock().expect(ErrFatal::LOCK).fen_read(Some(fen));
                if fen_result.is_err() {
                    self.comm.send(CommOut::Error(
                        ErrNormal::INCORRECT_FEN.to_string(),
                        fen.to_string(),
                    ));
                }

                // Restart analysis on the new position. (If the FEN-setup
                // failed, we restart on the old position.)
                if self.is_analyzing() {
                    search_params.search_mode = SearchMode::Infinite;
                    self.search.send(SearchControl::Start(search_params));
                }
            }

            // Accept an incoming usermove (and time control for the
            // engine's nex turn). Execute it on the board. Then react to
            // this usermove, according to the state we were in when the
            // move was received.
            XBoardIn::UserMove(m, tc) => {
                match self.state {
                    // When we are observing, we just execute the incoming
                    // move and send the game result (if any).
                    EngineState::Observing => {
                        if self.execute_move(m.clone()) {
                            self.send_game_result();
                        } else {
                            let im = CommOut::XBoard(XBoardOut::IllegalMove(m.clone()));
                            self.comm.send(im);
                        }
                    }

                    // When we're waiting, we execute the incoming move and
                    // then we start thinking if the game is not over. We
                    // only do this if we have a time control available.
                    EngineState::Waiting => {
                        if tc.is_set() {
                            if self.execute_move(m.clone()) {
                                if self.send_game_result() == GameEndReason::NotEnded {
                                    Engine::set_time_control(&mut search_params, tc);
                                    self.search.send(SearchControl::Start(search_params));
                                    self.set_thinking();
                                }
                            } else {
                                let im = CommOut::XBoard(XBoardOut::IllegalMove(m.clone()));
                                self.comm.send(im);
                            }
                        } else {
                            self.comm.send(CommOut::Error(
                                ErrNormal::TIME_CONTROL_NOT_SET.to_string(),
                                command.to_string(),
                            ));
                        }
                    }

                    // We can also accept a usermove during analysis mode.
                    EngineState::Analyzing => {
                        self.search.send(SearchControl::Abandon);
                        while self.info_rx() != Information::Search(SearchReport::Ready) {}

                        if self.execute_move(m.clone()) {
                            if self.send_game_result() == GameEndReason::NotEnded {
                                search_params.search_mode = SearchMode::Infinite;
                                self.search.send(SearchControl::Start(search_params));
                            } else {
                                self.set_observing();
                            }
                        } else {
                            let im = CommOut::XBoard(XBoardOut::IllegalMove(m.clone()));
                            self.comm.send(im);
                        }
                    }

                    // Do not accept user moves in other engine states.
                    _ => self.comm.send(CommOut::Error(
                        ErrNormal::COMMAND_INVALID.to_string(),
                        command.to_string(),
                    )),
                }
            }

            // Undo the last move.
            XBoardIn::Undo => {
                if self.is_thinking() {
                    self.search.send(SearchControl::Abandon);
                }

                self.board.lock().expect(ErrFatal::LOCK).unmake();
                self.is_waiting();
            }

            // Undo the last two moves (same side stays to move)
            XBoardIn::Remove => {
                if self.is_thinking() {
                    self.search.send(SearchControl::Abandon);
                }

                self.board.lock().expect(ErrFatal::LOCK).unmake();
                self.board.lock().expect(ErrFatal::LOCK).unmake();
                self.set_waiting();
            }

            XBoardIn::Result(result, reason) => {
                match self.state {
                    EngineState::Observing | EngineState::Waiting | EngineState::Thinking => {
                        if self.is_thinking() {
                            self.search.send(SearchControl::Abandon);
                        }
                        self.comm.send(CommOut::Message(format!(
                            "{}: result {}, reason {}",
                            Messages::ACCEPTED.to_string(),
                            result,
                            reason
                        )))
                    }
                    _ => {
                        self.comm.send(CommOut::Error(
                            ErrNormal::COMMAND_INVALID.to_string(),
                            command.to_string(),
                        ));
                    }
                };
            }

            // Send "Pong" to confirm that the engine is still active.
            XBoardIn::Ping(value) => self.comm.send(CommOut::XBoard(XBoardOut::Pong(*value))),

            // Enable sending search information.
            XBoardIn::Post => self.settings.verbosity = Verbosity::Full,

            // Disable sending search information.
            XBoardIn::NoPost => self.settings.verbosity = Verbosity::Silent,

            // Set the transposition table size.
            XBoardIn::Memory(mb) => self.tt_search.lock().expect(ErrFatal::LOCK).resize(*mb),

            // Start analyze mode.
            XBoardIn::Analyze => {
                if self.is_observing() || self.is_waiting() {
                    search_params.search_mode = SearchMode::Infinite;
                    self.search.send(SearchControl::Start(search_params));
                    self.set_analyzing();
                } else {
                    self.comm.send(CommOut::Error(
                        ErrNormal::COMMAND_INVALID.to_string(),
                        command.to_string(),
                    ));
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

    fn set_time_control(search_params: &mut SearchParams, tc: &TimeControl) {
        // In XBoard, "Depth" can stop the search even if other time
        //  controls are set, so this parameter is always set if higher
        //  than zero.
        if tc.depth() > 0 {
            search_params.depth = tc.depth();
        }

        // If we have a depth setting, but no settings for either MoveTime
        // or GameTime, we must be in Depth mode.
        if tc.depth() > 0 && !tc.is_move_time() && !tc.is_game_time() {
            search_params.search_mode = SearchMode::Depth
        }

        // MoveTime mode. (We can't be in GameTime at the same time.)
        if tc.is_move_time() {
            search_params.search_mode = SearchMode::MoveTime;
            search_params.move_time = tc.move_time();
        }

        // GameTime mode. (We can't be in MoveTime at the same time.)
        if tc.is_game_time() {
            search_params.search_mode = SearchMode::GameTime;
        }
    }
}
