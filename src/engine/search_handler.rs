use super::{
    defs::{ErrFatal, GameResult},
    Engine,
};
use crate::{comm::defs::CommOut, search::defs::SearchReport};

impl Engine {
    pub fn search_handler(&mut self, search_report: &SearchReport) {
        match search_report {
            SearchReport::Finished(m) => {
                let mut result: Option<GameResult> = None;

                if self.comm.info().requires_stateful_mode() {
                    if self.board.lock().expect(ErrFatal::LOCK).make(*m, &self.mg) {
                        if self.comm.info().requires_game_result() {
                            result = self.game_over();
                        }
                    } else {
                        panic!("{}", ErrFatal::GENERATED_ILLEGAL_MOVE);
                    }
                };

                self.comm.send(CommOut::BestMove(*m, result));
                self.set_waiting();
            }

            SearchReport::SearchCurrentMove(curr_move) => {
                self.comm.send(CommOut::SearchCurrMove(*curr_move));
            }

            SearchReport::SearchSummary(summary) => {
                self.comm.send(CommOut::SearchSummary(summary.clone()));
            }

            SearchReport::SearchStats(stats) => {
                self.comm.send(CommOut::SearchStats(*stats));
            }

            SearchReport::Ready => (),
        }
    }
}
