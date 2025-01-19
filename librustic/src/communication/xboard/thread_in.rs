use crate::basetypes::error::ErrFatal;
use crate::communication::defs::EngineInput;
use crate::communication::xboard::cmd_in::XBoardIn;
use crate::communication::xboard::XBoard;
use std::{io, sync::mpsc::Sender, thread};

impl XBoard {
    pub fn input_thread(&mut self, transmit_to_engine: Sender<EngineInput>) {
        let thread = thread::spawn(move || loop {
            let mut buffer = String::from("");
            io::stdin().read_line(&mut buffer).expect(ErrFatal::READ_IO);
            let cmd = XBoard::get_incoming_command(&buffer);
            let quit = cmd == XBoardIn::Quit;
            transmit_to_engine
                .send(EngineInput::XBoard(cmd))
                .expect(ErrFatal::HANDLE);

            // To prevent having to set up another channel (sending from
            // the engine to this thread) we'll have the input thread
            // terminate itself.
            if quit {
                break;
            }
        });

        // Store the thread handle.
        self.input_thread = Some(thread);
    }
}

impl XBoard {
    fn get_incoming_command(buffer: &str) -> XBoardIn {
        let input = buffer.trim_end().to_string();

        match input {
            cmd if cmd == "quit" => XBoardIn::Quit,
            _ => XBoardIn::Unknown(input),
        }
    }
}
