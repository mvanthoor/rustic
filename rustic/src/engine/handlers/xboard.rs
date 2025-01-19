use crate::engine::Engine;
use librustic::{
    basetypes::error::{ErrFatal, ErrUci},
    communication::{
        defs::EngineOutput,
        uci::cmd_out::UciOut,
        xboard::{cmd_in::XBoardIn, cmd_out::XBoardOut},
    },
};

// This is the UCI handler. It handles the incoming UCI commands and when
// needed, it sends replies to the communication module to be sent out of
// the engine to the GUI. Each enum variant of the UciIn type has a
// match-arm in this function.
impl Engine {
    pub fn xboard_handler(&mut self, command: XBoardIn) {
        match command {
            XBoardIn::Unknown(cmd) => self.xboard_unknown(cmd),
            XBoardIn::Quit => self.quit(),
        }
    }

    // This function handles commands that cannot be captured in one of the
    // UciIn enum variants; these are therefore unknown. These could be any
    // string of text. The engine can do whatever it wants with these; most
    //of the time they are ignored (except in debug mode, where an
    //InfoString is printed.) In Rustic, the function handles "board" as a
    // custom command that is not part of the UCI-specification. It may
    // handle other incoming commands as custom in the future as well.
    fn xboard_unknown(&self, cmd: String) {
        match cmd {
            c if c == "board" => {
                let board = format!("{}", self.board.lock().expect(ErrFatal::LOCK));
                self.comm.send(EngineOutput::Uci(UciOut::Custom(board)));
            }
            _ => {
                if self.debug {
                    let message = XBoardOut::Custom(format!("{}: {}", ErrUci::UNKNOWN_CMD, cmd));
                    self.comm.send(EngineOutput::XBoard(message));
                }
            }
        }
    }
}
