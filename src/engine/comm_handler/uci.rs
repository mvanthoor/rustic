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
    comm::{CommOutput, UciInput},
    defs::FEN_START_POSITION,
    engine::{
        defs::{EngineSetOption, ErrFatal, ErrNormal},
        Engine,
    },
    evaluation::Evaluation,
    search::defs::{SearchControl, SearchMode, SearchParams, SAFEGUARD},
};

impl Engine {
    pub fn uci_handler(&mut self, command: &UciInput) {
        // Setup default variables.
        let mut sp = SearchParams::new();
        sp.quiet = self.settings.quiet;

        match command {
            UciInput::Identification => self.comm.send(CommOutput::Identify),

            UciInput::NewGame => {
                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_read(Some(FEN_START_POSITION))
                    .expect(ErrFatal::NEW_GAME);
                self.tt_search.lock().expect(ErrFatal::LOCK).clear();
            }

            UciInput::IsReady => self.comm.send(CommOutput::Ready),

            UciInput::SetOption(option) => {
                match option {
                    EngineSetOption::Hash(value) => {
                        if let Ok(v) = value.parse::<usize>() {
                            self.tt_search.lock().expect(ErrFatal::LOCK).resize(v);
                        } else {
                            let msg = String::from(ErrNormal::NOT_INT);
                            self.comm.send(CommOutput::InfoString(msg));
                        }
                    }

                    EngineSetOption::ClearHash => {
                        self.tt_search.lock().expect(ErrFatal::LOCK).clear()
                    }

                    EngineSetOption::Nothing => (),
                };
            }

            UciInput::Position(fen, moves) => {
                let fen_result = self.board.lock().expect(ErrFatal::LOCK).fen_read(Some(fen));

                if fen_result.is_ok() {
                    for m in moves.iter() {
                        let ok = self.execute_move(m.clone());
                        if !ok {
                            let msg = format!("{}: {}", m, ErrNormal::NOT_LEGAL);
                            self.comm.send(CommOutput::InfoString(msg));
                            break;
                        }
                    }
                }

                if fen_result.is_err() {
                    let msg = ErrNormal::FEN_FAILED.to_string();
                    self.comm.send(CommOutput::InfoString(msg));
                }
            }

            UciInput::GoInfinite => {
                sp.search_mode = SearchMode::Infinite;
                self.search.send(SearchControl::Start(sp));
            }

            UciInput::GoDepth(depth) => {
                sp.depth = *depth;
                sp.search_mode = SearchMode::Depth;
                self.search.send(SearchControl::Start(sp));
            }

            UciInput::GoMoveTime(msecs) => {
                sp.move_time = *msecs - (SAFEGUARD as u128);
                sp.search_mode = SearchMode::MoveTime;
                self.search.send(SearchControl::Start(sp));
            }

            UciInput::GoNodes(nodes) => {
                sp.nodes = *nodes;
                sp.search_mode = SearchMode::Nodes;
                self.search.send(SearchControl::Start(sp));
            }

            UciInput::GoGameTime(gt) => {
                sp.game_time = *gt;
                sp.search_mode = SearchMode::GameTime;
                self.search.send(SearchControl::Start(sp));
            }

            UciInput::Stop => self.search.send(SearchControl::Stop),

            UciInput::Quit => self.quit(),

            // Custom commands
            UciInput::Board => self.comm.send(CommOutput::PrintBoard),

            UciInput::History => self.comm.send(CommOutput::PrintHistory),

            UciInput::Eval => {
                let mtx_board = &self.board.lock().expect(ErrFatal::LOCK);
                let eval = Evaluation::evaluate_position(mtx_board);
                let p_v = mtx_board.game_state.phase_value;
                let msg = format!("Evaluation: {} centipawns, phase value: {}", eval, p_v);
                self.comm.send(CommOutput::InfoString(msg));
            }

            UciInput::Help => self.comm.send(CommOutput::PrintHelp),
            UciInput::Unknown => (),
        }
    }
}
