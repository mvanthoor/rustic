use crate::{
    basetypes::error::ErrFatal,
    board::Board,
    communication::{
        defs::{IComm, Information},
        features::Features,
        protocol::Properties,
        uci::Uci,
    },
};
use std::sync::{mpsc::Sender, Arc, Mutex};

use super::cmd_out::UciOut;

impl IComm for Uci {
    fn init(
        &mut self,
        cmd_incoming_transmitter: Sender<Information>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<Features>>,
    ) {
        self.input_thread(cmd_incoming_transmitter);
        self.output_thread(board, options);
    }

    fn properties(&self) -> &Properties {
        &self.properties
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
