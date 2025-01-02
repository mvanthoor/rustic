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
    comm::defs::{EngineOptionDefaults, Information},
    search::{
        defs::{SearchControl, SearchData, SearchRefs, SearchReport, SearchSummary},
        transposition::TT,
    },
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
        Self::new(EngineOptionDefaults::HASH_DEFAULT)
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
