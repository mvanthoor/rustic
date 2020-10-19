use super::Engine;
use crate::{comm::CommControl, search::defs::SearchReport};

impl Engine {
    pub fn search_reports(&mut self, search_report: &SearchReport) {
        match search_report {
            SearchReport::Finished(m) => {
                self.comm.send(CommControl::PrintBestMove(*m));
                self.comm.send(CommControl::Update);
            }

            SearchReport::Summary(summary) => {
                self.comm.send(CommControl::PrintSearchSummary(*summary));
            }
        }
    }
}
