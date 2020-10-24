use super::Search;

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
}
