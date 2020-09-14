use super::IComm;
// use crate::{board::Board, movegen::MoveGenerator};

pub struct Uci {}

impl Uci {
    pub fn new() -> Self {
        Self {}
    }
}

impl IComm for Uci {
    fn start(&self) {
        println!("UCI communication.");
    }
}
