use crate::{
    basetypes::error::ErrFatal,
    communication::{
        defs::Information,
        uci::{cmd_in::UciIn, parse, Uci},
    },
};
use std::{io, sync::mpsc::Sender, thread};

impl Uci {
    pub fn input_thread(&mut self, transmit_to_engine: Sender<Information>) {
        let thread = thread::spawn(move || loop {
            let mut buffer = String::from("");
            io::stdin().read_line(&mut buffer).expect(ErrFatal::READ_IO);
            let cmd = Uci::get_incoming_command(&buffer);
            let quit = cmd == UciIn::Quit;
            let info = Information::Command(cmd);
            transmit_to_engine.send(info).expect(ErrFatal::HANDLE);

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

impl Uci {
    fn get_incoming_command(buffer: &str) -> UciIn {
        let input = buffer.trim_end().to_string();

        match input {
            cmd if cmd == "uci" => UciIn::Uci,
            cmd if cmd == "isready" => UciIn::IsReady,
            cmd if cmd == "ucinewgame" => UciIn::UciNewGame,
            cmd if cmd == "debug on" => UciIn::DebugOn,
            cmd if cmd == "debug off" => UciIn::DebugOff,
            cmd if cmd == "stop" => UciIn::Stop,
            cmd if cmd == "quit" => UciIn::Quit,
            cmd if cmd == "board" => UciIn::Board,
            cmd if cmd.starts_with("position") => parse::position(&cmd),
            cmd if cmd.starts_with("go") => parse::go(&cmd),
            cmd if cmd.starts_with("setoption") => parse::setoption(&cmd),
            _ => UciIn::Unknown(input),
        }
    }
}
