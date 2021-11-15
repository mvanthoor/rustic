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
    comm::defs::{CommOut, UciIn, UciOut},
    defs::FEN_START_POSITION,
    engine::{
        defs::{EngineSetOption, ErrFatal, ErrNormal},
        Engine,
    },
    search::defs::{SearchControl, SearchMode, SearchParams, SAFEGUARD},
};

impl Engine {
    pub fn uci_handler(&mut self, command: &UciIn) {
        // Setup default variables.
        let mut sp = SearchParams::new();
        sp.verbosity = self.settings.verbosity;

        match command {
            UciIn::Uci => self.comm.send(CommOut::Uci(UciOut::Identify)),

            UciIn::UciNewGame => {
                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_read(Some(FEN_START_POSITION))
                    .expect(ErrFatal::NEW_GAME);
                self.tt_search.lock().expect(ErrFatal::LOCK).clear();
            }

            UciIn::IsReady => self.comm.send(CommOut::Uci(UciOut::Ready)),

            UciIn::SetOption(option) => {
                match option {
                    EngineSetOption::Hash(value) => {
                        if let Ok(v) = value.parse::<usize>() {
                            self.tt_search.lock().expect(ErrFatal::LOCK).resize(v);
                        } else {
                            self.comm.send(CommOut::Error(
                                ErrNormal::NOT_INT.to_string(),
                                value.to_string(),
                            ));
                        }
                    }

                    EngineSetOption::ClearHash => {
                        self.tt_search.lock().expect(ErrFatal::LOCK).clear()
                    }

                    EngineSetOption::Nothing => (),
                };
            }

            UciIn::Position(fen, moves) => {
                let fen_result = self.board.lock().expect(ErrFatal::LOCK).fen_read(Some(fen));

                if fen_result.is_ok() {
                    for m in moves.iter() {
                        let ok = self.execute_move(m.clone());
                        if !ok {
                            let err = ErrNormal::NOT_LEGAL.to_string();
                            self.comm.send(CommOut::Error(err, m.clone()));
                            break;
                        }
                    }
                }

                if fen_result.is_err() {
                    self.comm.send(CommOut::Error(
                        ErrNormal::FEN_FAILED.to_string(),
                        fen.clone(),
                    ));
                }
            }

            UciIn::GoInfinite => {
                sp.search_mode = SearchMode::Infinite;
                self.search.send(SearchControl::Start(sp));
                self.set_analyzing();
            }

            UciIn::GoDepth(depth) => {
                sp.depth = *depth;
                sp.search_mode = SearchMode::Depth;
                self.search.send(SearchControl::Start(sp));
                self.set_thinking();
            }

            UciIn::GoMoveTime(msecs) => {
                sp.move_time = *msecs - (SAFEGUARD as u128);
                sp.search_mode = SearchMode::MoveTime;
                self.search.send(SearchControl::Start(sp));
                self.set_thinking();
            }

            UciIn::GoNodes(nodes) => {
                sp.nodes = *nodes;
                sp.search_mode = SearchMode::Nodes;
                self.search.send(SearchControl::Start(sp));
                self.set_thinking();
            }

            UciIn::GoGameTime(gt) => {
                sp.game_time = *gt;
                sp.search_mode = SearchMode::GameTime;
                self.search.send(SearchControl::Start(sp));
                self.set_thinking();
            }

            UciIn::Stop => {
                self.search.send(SearchControl::Stop);
                self.set_waiting();
            }
        }
    }
}
