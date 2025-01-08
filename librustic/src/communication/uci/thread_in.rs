use crate::{
    basetypes::error::ErrFatal, communication::uci::cmd_in::UciIn, communication::uci::Uci,
};
use std::{io, sync::mpsc::Sender, thread};

impl Uci {
    pub fn input_thread(&mut self, to_engine_thread: Sender<UciIn>) {
        let mut buffer_incoming_cmd = String::from("");

        let thread = thread::spawn(move || loop {
            io::stdin()
                .read_line(&mut buffer_incoming_cmd)
                .expect(ErrFatal::READ_IO);
            let cmd = Uci::get_input(&buffer_incoming_cmd);
            to_engine_thread.send(cmd).expect(ErrFatal::HANDLE);
            buffer_incoming_cmd = String::from("");

            if cmd == UciIn::Quit {
                break;
            }
        });

        // Store the thread handle.
        self.input_thread = Some(thread);
    }
}

impl Uci {
    fn get_input(input: &str) -> UciIn {
        let input = input.trim_end().to_string();

        match input {
            cmd if cmd == "uci" => UciIn::Uci,
            _ => UciIn::Unknown,
        }
    }
}
