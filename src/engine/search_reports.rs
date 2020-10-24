use super::Engine;
use crate::{comm::CommControl, search::defs::SearchReport};

impl Engine {
    pub fn search_reports(&mut self, search_report: &SearchReport) {
        match search_report {
            SearchReport::Finished(m) => {
                self.comm.send(CommControl::BestMove(*m));
                self.comm.send(CommControl::Update);
            }

            SearchReport::SearchCurrentMove(current) => {
                self.comm.send(CommControl::SearchCurrent(*current));
            }

            SearchReport::SearchSummary(summary) => {
                self.comm.send(CommControl::SearchSummary(*summary));
            }

            SearchReport::SearchStats(stats) => {
                self.comm.send(CommControl::SearchStats(*stats));
            }
        }
    }
}
