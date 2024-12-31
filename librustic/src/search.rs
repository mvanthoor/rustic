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
    search::defs::SearchData,
    search::defs::{SearchControl, SearchRefs, SearchReport, SearchSummary},
    search::transposition::TT,
};
use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    thread::JoinHandle,
};

pub struct Search {
    handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<SearchControl>>,
    transposition: Arc<Mutex<TT<SearchData>>>,
}

impl Default for Search {
    fn default() -> Self {
        Self::new(32)
    }
}

impl Search {
    pub fn new(tt_size: usize) -> Self {
        Self {
            handle: None,
            control_tx: None,
            transposition: Arc::new(Mutex::new(TT::<SearchData>::new(tt_size))),
        }
    }
}
