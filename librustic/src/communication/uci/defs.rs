use crate::{
    basetypes::error::ErrFatal,
    board::Board,
    communication::defs::Features,
    communication::uci::{cmd_in::UciIn, cmd_out::UciOut},
    search::defs::SearchReport,
    {communication::defs::IComm, defs::About},
};
use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    thread::JoinHandle,
};

pub struct Uci {
    pub about: About,
    pub input_thread: Option<JoinHandle<()>>,
    pub output_thread: Option<JoinHandle<()>>,
    pub uci_output: Option<Sender<UciOut>>,
}

impl Default for Uci {
    fn default() -> Self {
        Self::new(About::default())
    }
}

// Public functions
impl Uci {
    // Create a new console.
    pub fn new(about: About) -> Self {
        Self {
            about,
            input_thread: None,
            output_thread: None,
            uci_output: None,
        }
    }
}

impl IComm for Uci {
    fn init(
        &mut self,
        cmd_in_tx: Sender<UciIn>,
        search_tx: Sender<SearchReport>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<Features>>,
    ) {
        self.input_thread(cmd_in_tx);
        self.output_thread(board, options);
    }

    // The engine thread can use this function to send information out of
    // the engine towards a GUI. Effectively the output thread will print
    // the information to stdout.
    fn send(&self, info: UciOut) {
        if let Some(out) = &self.uci_output {
            out.send(info).expect(ErrFatal::CHANNEL);
        }
    }

    // The engine thread will send 'quit' to the communication threads and
    // then wait here until shutdown is completed.
    fn shutdown(&mut self) {
        if let Some(h) = self.input_thread.take() {
            h.join().expect(ErrFatal::THREAD);
        }

        if let Some(h) = self.output_thread.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }
}
