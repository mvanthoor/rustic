use crate::{
    comm::defs::{CommOut, UciIn, UciOut},
    defs::FEN_START_POSITION,
    engine::defs::{EngineSetOption, ErrFatal, ErrNormal},
    engine::Engine,
    search::defs::{SearchControl, SearchMode, SearchParams, SAFEGUARD},
};

impl Engine {
    pub fn uci_handler(&mut self, command: &UciIn) {
        // Setup default variables.
        let mut search_params = SearchParams::new();
        search_params.verbosity = self.settings.verbosity;

        match command {
            UciIn::Uci => self.comm.send(CommOut::Uci(UciOut::Identify)),

            UciIn::UciNewGame => {
                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .read_fen(Some(FEN_START_POSITION))
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
                            self.comm
                                .send(CommOut::Error(ErrNormal::NOT_INT, value.to_string()));
                        }
                    }

                    EngineSetOption::ClearHash => {
                        self.tt_search.lock().expect(ErrFatal::LOCK).clear()
                    }

                    EngineSetOption::Nothing => (),
                };
            }

            UciIn::Position(fen, moves) => {
                let fen_result = self.board.lock().expect(ErrFatal::LOCK).read_fen(Some(fen));

                if fen_result.is_ok() {
                    for m in moves.iter() {
                        let ok = self.execute_move(m.clone());
                        if !ok {
                            self.comm
                                .send(CommOut::Error(ErrNormal::NOT_LEGAL, m.clone()));
                            break;
                        }
                    }
                }

                if fen_result.is_err() {
                    self.comm
                        .send(CommOut::Error(ErrNormal::FEN_FAILED, fen.clone()));
                }
            }

            UciIn::GoInfinite => {
                search_params.search_mode = SearchMode::Infinite;
                self.search.send(SearchControl::Start(search_params));
                self.set_analyzing();
            }

            UciIn::GoDepth(depth) => {
                search_params.depth = *depth;
                search_params.search_mode = SearchMode::Depth;
                self.search.send(SearchControl::Start(search_params));
                self.set_thinking();
            }

            UciIn::GoMoveTime(msecs) => {
                search_params.move_time = *msecs - (SAFEGUARD as u128);
                search_params.search_mode = SearchMode::MoveTime;
                self.search.send(SearchControl::Start(search_params));
                self.set_thinking();
            }

            UciIn::GoNodes(nodes) => {
                search_params.nodes = *nodes;
                search_params.search_mode = SearchMode::Nodes;
                self.search.send(SearchControl::Start(search_params));
                self.set_thinking();
            }

            UciIn::GoGameTime(gt) => {
                search_params.game_time = *gt;
                search_params.search_mode = SearchMode::GameTime;
                self.search.send(SearchControl::Start(search_params));
                self.set_thinking();
            }

            UciIn::Stop => {
                self.search.send(SearchControl::Stop);
                self.set_waiting();
            }
        }
    }
}
