use super::IComm;
// use crate::{board::Board, movegen::MoveGenerator};

pub struct Xboard {}

impl Xboard {
    pub fn new() -> Self {
        Self {}
    }
}

impl IComm for Xboard {
    fn start(&self) {
        println!("XBoard communication.");
    }
}
