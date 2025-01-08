use crate::{
    basetypes::error::ErrFatal, communication::uci::cmd_in::UciIn, communication::uci::Uci,
};
use std::{io, sync::mpsc::Sender, thread};

impl Uci {
    pub fn input_thread(&mut self, transmit_to_engine: Sender<UciIn>) {
        let thread = thread::spawn(move || loop {
            let mut buffer = String::from("");
            io::stdin().read_line(&mut buffer).expect(ErrFatal::READ_IO);
            let cmd = Uci::get_incoming_cmd(&buffer);
            transmit_to_engine.send(cmd).expect(ErrFatal::HANDLE);

            // To prevent having to set up another channel (sending from
            // the engine to this thread) we'll have the input thread
            // terminate itself.
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
