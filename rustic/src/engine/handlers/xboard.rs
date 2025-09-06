use crate::engine::Engine;
use librustic::{
    basetypes::error::{ErrFatal, ErrXboard},
    communication::{
        defs::EngineOutput,
        xboard::{cmd_in::XBoardIn, cmd_out::XBoardOut},
    },
    defs::FEN_START_POSITION,
};

// This is the UCI handler. It handles the incoming UCI commands and when
// needed, it sends replies to the communication module to be sent out of
// the engine to the GUI. Each enum variant of the UciIn type has a
// match-arm in this function.
impl Engine {
    pub fn xboard_handler(&mut self, command: XBoardIn) {
        match command {
            XBoardIn::XBoard => self.comm.send(EngineOutput::XBoard(XBoardOut::NewLine)),
            XBoardIn::Protover(version) => {
                if version != 2 {
                    let error = ErrXboard::NOT_PROTOVER_2.to_string();
                    let message = XBoardOut::Error(error, format!("{version}"));
                    self.comm.send(EngineOutput::XBoard(message));
                } else {
                    self.comm
                        .send(EngineOutput::XBoard(XBoardOut::XboardFeatures));
                }
            }
            XBoardIn::New => {
                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_setup(Some(FEN_START_POSITION))
                    .expect(ErrFatal::NEW_GAME);
                self.search.transposition_clear();
            }
            XBoardIn::Quit => self.quit(),
            XBoardIn::Unknown(cmd) => self.xboard_unknown(cmd),
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
        if self.debug {
            let message = XBoardOut::Custom(format!("{}: {}", ErrXboard::UNKNOWN_CMD, cmd));
            self.comm.send(EngineOutput::XBoard(message));
        }
    }
}
