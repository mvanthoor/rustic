use super::{
    defs::{
        SearchControl, SearchCurrentMove, SearchRefs, SearchReport, SearchStats, SearchTerminate,
    },
    Search,
};
use crate::{
    engine::defs::{ErrFatal, Information},
    movegen::defs::Move,
};

impl Search {
    // This function calculates the number of nodes per second.
    pub fn nodes_per_second(nodes: usize, msecs: u128) -> usize {
        let mut nps: usize = 0;
        let seconds = msecs as f64 / 1000f64;
        if seconds > 0f64 {
            nps = (nodes as f64 / seconds).round() as usize;
        }
        nps
    }

    // Returns true if the current recursive iteration of alpha_beta is at
    // the root position.
    pub fn is_root(current_depth: u8, ab_depth: u8) -> bool {
        current_depth == ab_depth
    }

    // This function sends the currently searched move to the engine thread.
    pub fn send_updated_current_move(refs: &mut SearchRefs, current: Move, count: u8) {
        let scm = SearchCurrentMove::new(current, count);
        let scm_report = SearchReport::SearchCurrentMove(scm);
        let information = Information::Search(scm_report);
        refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
    }

    // This function sends updated search statistics to the engine thread.
    pub fn send_updated_stats(refs: &mut SearchRefs) {
        let milli_seconds = refs.search_info.start_time.elapsed().as_millis();
        let nps = Search::nodes_per_second(refs.search_info.nodes, milli_seconds);
        let stats = SearchStats::new(refs.search_info.nodes, nps);
        let stats_report = SearchReport::SearchStats(stats);
        let information = Information::Search(stats_report);
        refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
        refs.search_info.last_stats = refs.search_info.nodes;
    }

    // This function checks termination conditions and sets the termination
    // flag if this is required.
    pub fn check_for_termination(refs: &mut SearchRefs) {
        // Terminate search if stop or quit command is received.
        let cmd = refs.control_rx.try_recv().unwrap_or(SearchControl::Nothing);
        match cmd {
            SearchControl::Stop => refs.search_info.terminate = SearchTerminate::Stop,
            SearchControl::Quit => refs.search_info.terminate = SearchTerminate::Quit,
            _ => (),
        };

        // Terminate search if allowed time for this move has run out.
        let elapsed = refs.search_info.start_time.elapsed().as_millis();
        let time_up = elapsed >= refs.search_params.time_for_move;
        if time_up {
            refs.search_info.terminate = SearchTerminate::Stop
        }

        // Update last checkpoint
        refs.search_info.last_checkpoint = refs.search_info.nodes;
    }
}
