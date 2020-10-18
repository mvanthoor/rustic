use super::Engine;
use crate::search::defs::SearchReport;

impl Engine {
    pub fn search_reports(&mut self, search_report: &SearchReport) {
        match search_report {
            SearchReport::Finished => {
                println!("Search done.");
            }
        }
    }
}
