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
                    let msg = XBoardOut::Error(error, format!("{version}"));
                    self.comm.send(EngineOutput::XBoard(msg));
                } else {
                    self.comm.send(EngineOutput::XBoard(XBoardOut::Features));
                }
            }
            XBoardIn::Ping(n) => self.comm.send(EngineOutput::XBoard(XBoardOut::Pong(n))),
            XBoardIn::New => {
                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_setup(Some(FEN_START_POSITION))
                    .expect(ErrFatal::NEW_GAME);
                self.search.transposition_clear();
            }
            XBoardIn::SetBoard(fen) => {
                let fen_result = self
                    .board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_setup(Some(fen.as_str()));
                if fen_result.is_err() && self.debug {
                    let error = ErrXboard::FEN_ERROR.to_string();
                    let message = XBoardOut::Error(error, fen);
                    self.comm.send(EngineOutput::XBoard(message));
                }
            }
            XBoardIn::Quit => self.quit(),

            // Custom commands
            XBoardIn::DebugOn => self.debug = true,
            XBoardIn::DebugOff => self.debug = false,
            XBoardIn::Board => {
                let print_board = format!("{}", self.board.lock().expect(ErrFatal::LOCK));
                let message = XBoardOut::Custom(print_board);
                self.comm.send(EngineOutput::XBoard(message));
            }
            XBoardIn::Unknown(cmd) => self.xboard_unknown(cmd),
        }
    }

    fn xboard_unknown(&self, cmd: String) {
        if self.debug {
            let error = String::from(ErrXboard::UNKNOWN_CMD);
            let out = XBoardOut::Error(error, cmd);
            self.comm.send(EngineOutput::XBoard(out));
        }
    }
}
