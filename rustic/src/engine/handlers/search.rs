use crate::engine::Engine;
use librustic::{
    communication::{defs::EngineOutput, uci::cmd_out::UciOut},
    search::defs::SearchReport,
};

impl Engine {
    pub fn search_handler(&mut self, search_report: SearchReport) {
        match search_report {
            SearchReport::Ready => (),
            SearchReport::Finished(bestmove) => self
                .comm
                .send(EngineOutput::Uci(UciOut::BestMove(bestmove))),
            SearchReport::SearchCurrentMove(curr_move) => self
                .comm
                .send(EngineOutput::Uci(UciOut::SearchCurrMove(curr_move))),
            SearchReport::SearchSummary(summary) => self
                .comm
                .send(EngineOutput::Uci(UciOut::SearchSummary(summary))),
            SearchReport::SearchStats(stats) => self
                .comm
                .send(EngineOutput::Uci(UciOut::SearchStats(stats))),
        }
    }
}
