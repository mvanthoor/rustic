use crate::{
    basetypes::error::ErrFatal, communication::uci::cmd_in::UciIn, communication::uci::Uci,
};
use std::{io, sync::mpsc::Sender, thread};

impl Uci {
    pub fn input_thread(&mut self, transmit_to_engine: Sender<UciIn>) {
        let mut buffer_incoming_cmd = String::from("");

        let thread = thread::spawn(move || loop {
            io::stdin()
                .read_line(&mut buffer_incoming_cmd)
                .expect(ErrFatal::READ_IO);
            let cmd = Uci::get_incoming_cmd(&buffer_incoming_cmd);
            transmit_to_engine.send(cmd).expect(ErrFatal::HANDLE);
            buffer_incoming_cmd = String::from("");

            // To prevent having to set up a send/receive channel from the
            // engine to this thread for only this single command, we will
            // have the input thread quit itself.
            if cmd == UciIn::Quit {
                break;
            }
        });

        // Store the thread handle.
        self.input_thread = Some(thread);
    }
}

impl Uci {
    fn get_incoming_cmd(buffer: &str) -> UciIn {
        let input = buffer.trim_end().to_string();

        match input {
            cmd if cmd == "uci" => UciIn::Uci,
            _ => UciIn::Unknown,
        }
    }
}
