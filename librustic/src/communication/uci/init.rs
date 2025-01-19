use crate::{
    basetypes::error::ErrFatal,
    communication::{
        defs::{EngineInput, EngineOutput, IComm},
        feature::Feature,
        protocol::Properties,
        uci::Uci,
    },
};
use std::sync::{mpsc::Sender, Arc};

impl IComm for Uci {
    fn init(&mut self, cmd_incoming_transmitter: Sender<EngineInput>, options: Arc<Vec<Feature>>) {
        self.input_thread(cmd_incoming_transmitter);
        self.output_thread(options);
    }

    fn properties(&self) -> &Properties {
        &self.properties
    }

    // The engine thread can use this function to send information out of
    // the engine towards a GUI. Effectively the output thread will print
    // the information to stdout.
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
