use crate::basetypes::error::ErrFatal;
use crate::communication::{defs::EngineInput, xboard::XBoard, xboard::cmd_in::XBoardIn};

use std::{io, sync::mpsc::Sender, thread};

use super::parse;

impl XBoard {
    pub fn tread_in(&mut self, transmit_to_engine: Sender<EngineInput>) {
        let thread = thread::spawn(move || {
            loop {
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
            cmd if cmd == "xboard" => XBoardIn::XBoard,
            cmd if cmd == "new" => XBoardIn::New,
            cmd if cmd == "quit" => XBoardIn::Quit,
            cmd if cmd == "board" => XBoardIn::Board,

            // See the KEYS constant in xboard-defs for an array of
            // commands which are key-value pairs.
            cmd if cmd.starts_with("protover") => parse::key_value_pair(&cmd),
            cmd if cmd.starts_with("ping") => parse::key_value_pair(&cmd),

            // Custom commands
            cmd if cmd == "debug on" => XBoardIn::DebugOn,
            cmd if cmd == "debug off" => XBoardIn::DebugOff,
            _ => XBoardIn::Unknown(input),
        }
    }
}
