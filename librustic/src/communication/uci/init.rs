use crate::{
    basetypes::error::ErrFatal,
    communication::{
        defs::{EngineInput, EngineOutput, IComm},
        feature::Feature,
        protocol::Properties,
        uci::Uci,
    },
};
use std::sync::{Arc, mpsc::Sender};

impl IComm for Uci {
    fn init(&mut self, cmd_incoming_tx: Sender<EngineInput>, options: Arc<Vec<Feature>>) {
        self.input_thread(cmd_incoming_tx);
        self.output_thread(options);
    }

    fn properties(&self) -> &Properties {
        &self.properties
    }

    // The engine thread can use this function to put information into the
    // output thread, which will then print it to stdout.
    fn send(&self, info: EngineOutput) {
        if let Some(out) = &self.output_write {
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
