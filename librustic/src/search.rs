// search.rs contains the engine's search routine.

mod alpha_beta;
mod control;
pub mod defs;
mod iter_deep;
mod qsearch;
mod sorting;
mod time;
mod transposition;
mod utils;

use crate::{
    engine::defs::{ErrFatal, Information},
    search::defs::{SearchControl, SearchRefs, SearchReport, SearchSummary},
};
use std::{sync::mpsc::Sender, thread::JoinHandle};

pub struct Search {
    handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<SearchControl>>,
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

impl Search {
    pub fn new() -> Self {
        Self {
            handle: None,
            control_tx: None,
        }
    }
}
