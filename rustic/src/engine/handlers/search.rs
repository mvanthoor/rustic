use crate::engine::Engine;
use librustic::{communication::uci::cmd_out::UciOut, search::defs::SearchReport};

impl Engine {
    pub fn search_handler(&mut self, search_report: SearchReport) {
        match search_report {
            SearchReport::Ready => (),
            SearchReport::Finished(bestmove) => self.comm.send(UciOut::BestMove(bestmove)),
            SearchReport::SearchCurrentMove(curr_move) => {
                self.comm.send(UciOut::SearchCurrMove(curr_move))
            }
            SearchReport::SearchSummary(summary) => self.comm.send(UciOut::SearchSummary(summary)),
            SearchReport::SearchStats(stats) => self.comm.send(UciOut::SearchStats(stats)),
        }
    }
}
