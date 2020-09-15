pub mod console;
// pub mod uci;
// pub mod xboard;

use crate::board::Board;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub trait IComm {
    fn start(&self, board: Arc<Mutex<Board>>) -> JoinHandle<()>;
}
