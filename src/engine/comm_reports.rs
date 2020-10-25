use super::{
    defs::{ErrFatal, ErrNormal},
    Engine,
};
use crate::{
    comm::{uci::UciReport, CommControl, CommReport},
    evaluation::evaluate_position,
    search::defs::{SearchControl, SearchMode, SearchParams},
};

// This block implements handling of incoming information, which will be in
// the form of either Comm or Search reports.
impl Engine {
    pub fn comm_reports(&mut self, comm_report: &CommReport) {
        // Split out the comm reports according to their source.
        match comm_report {
            CommReport::Uci(u) => self.comm_reports_uci(u),
        }
    }

    // Handles "Uci" Comm reports sent by the UCI-module.
    fn comm_reports_uci(&mut self, u: &UciReport) {
        match u {
            // Uci commands
            UciReport::Uci => self.comm.send(CommControl::Identify),
            UciReport::IsReady => self.comm.send(CommControl::Ready),
            UciReport::Position(fen, moves) => {
                let fen_result = self.board.lock().expect(ErrFatal::LOCK).fen_read(Some(fen));

                if fen_result.is_ok() {
                    for m in moves.iter() {
                        let ok = self.execute_move(m.clone());
                        if !ok {
                            let msg = format!("{}: {}", m, ErrNormal::NOT_LEGAL);
                            self.comm.send(CommControl::InfoString(msg));
                            break;
                        }
                    }
                }

                if fen_result.is_err() {
                    let msg = ErrNormal::FEN_FAILED.to_string();
                    self.comm.send(CommControl::InfoString(msg));
                }
            }

            UciReport::GoInfinite => {
                let sp = SearchParams::new(0, 0, 0, SearchMode::Infinite);
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::GoDepth(depth) => {
                let sp = SearchParams::new(*depth, 0, 0, SearchMode::Depth);
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::GoMoveTime(milliseconds) => {
                let sp = SearchParams::new(0, *milliseconds, 0, SearchMode::MoveTime);
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::GoNodes(nodes) => {
                let sp = SearchParams::new(0, 0, *nodes, SearchMode::Nodes);
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::Stop => self.search.send(SearchControl::Stop),
            UciReport::Quit => self.quit(),

            // Custom commands
            UciReport::Board => self.comm.send(CommControl::PrintBoard),
            UciReport::Eval => {
                let e = evaluate_position(&self.board.lock().expect(ErrFatal::LOCK));
                let msg = format!("Evaluation: {} centipawns", e);
                self.comm.send(CommControl::InfoString(msg));
            }
            UciReport::Help => self.comm.send(CommControl::PrintHelp),
            UciReport::Unknown => (),
        }
    }
}
