// search.rs contains the engine's search routine.

use crate::{board::Board, engine::defs::Information};
use crossbeam_channel::Sender;
use std::sync::{Arc, Mutex};

pub struct Search {}

impl Search {
    pub fn new() -> Self {
        Self {}
    }

    pub fn init(&mut self, report_tx: Sender<Information>, board: Arc<Mutex<Board>>) {}
}
