pub mod console;
// pub mod uci;
// pub mod xboard;

use std::thread::JoinHandle;

pub trait IComm {
    fn start(&self) -> JoinHandle<()>;
}
