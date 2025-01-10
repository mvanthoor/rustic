use crate::engine::Engine;
use librustic::{
    basetypes::error::{ErrFatal, ErrNormal},
    communication::uci::{cmd_in::UciIn, cmd_out::UciOut},
    defs::FEN_START_POSITION,
    search::defs::{SearchControl, SearchMode, SearchParams, SAFEGUARD},
};

const UNKNOWN: &str = "Unknown command";

// This block implements handling of incoming information, which will be in
// the form of either Comm or Search reports.
impl Engine {
    pub fn comm_handler(&mut self, command: UciIn) {
        let mut search_params = SearchParams::new();
        search_params.verbosity = self.settings.verbosity;

        match command {
            UciIn::Uci => self.comm.send(UciOut::Id),
            UciIn::IsReady => self.comm.send(UciOut::ReadyOk),
            UciIn::UciNewGame => {
                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_setup(Some(FEN_START_POSITION))
                    .expect(ErrFatal::NEW_GAME);
                self.search.transposition_clear();
            }
            UciIn::Position(fen, moves) => {
                let fen_result = self
                    .board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_setup(Some(fen.as_str()));

                if fen_result.is_ok() {
                    for m in moves.iter() {
                        if !self.execute_move(m) && self.debug {
                            let fail = format!("{}: {}", ErrNormal::NOT_LEGAL, m.clone());
                            self.comm.send(UciOut::InfoString(fail));
                            break;
                        }
                    }
                } else if self.debug {
                    let fail = format!("{}: {}", ErrNormal::FEN_FAILED, fen.clone());
                    self.comm.send(UciOut::InfoString(fail));
                }
            }
            UciIn::GoInfinite => {
                search_params.search_mode = SearchMode::Infinite;
                self.search.send(SearchControl::Start(search_params));
            }
            UciIn::GoDepth(depth) => {
                search_params.depth = depth;
                search_params.search_mode = SearchMode::Depth;
                self.search.send(SearchControl::Start(search_params));
            }
            UciIn::GoMoveTime(msecs) => {
                search_params.move_time = msecs - (SAFEGUARD as u128);
                search_params.search_mode = SearchMode::MoveTime;
                self.search.send(SearchControl::Start(search_params));
            }
            UciIn::GoNodes(nodes) => {
                search_params.nodes = nodes;
                search_params.search_mode = SearchMode::Nodes;
                self.search.send(SearchControl::Start(search_params));
            }
            UciIn::GoGameTime(gt) => {
                search_params.game_time = gt;
                search_params.search_mode = SearchMode::GameTime;
                self.search.send(SearchControl::Start(search_params));
            }
            UciIn::DebugOn => self.debug = true,
            UciIn::DebugOff => self.debug = false,
            UciIn::Stop => self.search.send(SearchControl::Stop),
            UciIn::Quit => self.quit(),
            UciIn::Unknown(cmd) => {
                if self.debug {
                    self.comm
                        .send(UciOut::InfoString(format!("{UNKNOWN}: {cmd}")));
                }
            }
            UciIn::Board => self.comm.send(UciOut::PrintBoard),
        }
    }
}
