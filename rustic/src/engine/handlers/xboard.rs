use crate::engine::Engine;
use librustic::{
    basetypes::error::{ErrFatal, ErrXboard},
    communication::{
        defs::{EngineOutput, EngineState},
        xboard::{cmd_in::XBoardIn, cmd_out::XBoardOut},
    },
    defs::FEN_START_POSITION,
    search::defs::{SearchControl, SearchMode, SearchParams},
};

impl Engine {
    pub fn xboard_handler(&mut self, command: XBoardIn) {
        let mut search_params = SearchParams::new();
        search_params.verbosity = self.settings.verbosity;

        match command {
            XBoardIn::XBoard => {
                self.comm.send(EngineOutput::XBoard(XBoardOut::NewLine));
                self.set_observing();
            }
            XBoardIn::Protover(version) => {
                if version != 2 {
                    let error = ErrXboard::NOT_PROTOVER_2.to_string();
                    let message = XBoardOut::Error(error, format!("{version}"));
                    self.comm.send(EngineOutput::XBoard(message));
                } else {
                    self.comm.send(EngineOutput::XBoard(XBoardOut::Features));
                }
            }
            XBoardIn::Ping(n) => self.comm.send(EngineOutput::XBoard(XBoardOut::Pong(n))),
            XBoardIn::New => {
                if self.is_observing() {
                    self.board
                        .lock()
                        .expect(ErrFatal::LOCK)
                        .fen_setup(Some(FEN_START_POSITION))
                        .expect(ErrFatal::NEW_GAME);
                    self.search.transposition_clear();
                } else if self.debug {
                    let msg = Engine::inapplicable_command(XBoardIn::New);
                    self.comm.send(EngineOutput::XBoard(msg));
                }
            }
            XBoardIn::Usermove(m) => match self.get_state() {
                EngineState::Analyzing => {
                    self.search.send(SearchControl::Abandon);
                    if !self.execute_move(&m) {
                        self.comm
                            .send(EngineOutput::XBoard(XBoardOut::IllegalMove(m)));
                    }
                    search_params.search_mode = SearchMode::Infinite;
                    self.search.send(SearchControl::Start(search_params));
                }
                EngineState::Observing => {
                    if !self.execute_move(&m) {
                        self.comm
                            .send(EngineOutput::XBoard(XBoardOut::IllegalMove(m)));
                    }
                }
                _ => {
                    let msg = Engine::inapplicable_command(XBoardIn::Usermove(m));
                    self.comm.send(EngineOutput::XBoard(msg));
                }
            },
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
            XBoardIn::Analyze => {
                if self.is_observing() {
                    self.set_analyzing();
                    search_params.search_mode = SearchMode::Infinite;
                    self.search.send(SearchControl::Start(search_params));
                } else if self.debug {
                    let msg = Engine::inapplicable_command(XBoardIn::Analyze);
                    self.comm.send(EngineOutput::XBoard(msg));
                }
            }
            XBoardIn::Exit => {
                if self.is_analyzing() {
                    self.search.send(SearchControl::Abandon);
                    self.set_observing();
                } else if self.debug {
                    let msg = Engine::inapplicable_command(XBoardIn::Exit);
                    self.comm.send(EngineOutput::XBoard(msg));
                }
            }
            XBoardIn::Force => {
                if self.is_thinking() {
                    self.search.send(SearchControl::Abandon);
                    self.set_observing();
                } else if self.debug {
                    let msg = Engine::inapplicable_command(XBoardIn::Force);
                    self.comm.send(EngineOutput::XBoard(msg));
                }
            }
            XBoardIn::Quit => self.quit(),

            // Custom commands
            XBoardIn::State => {
                let state = format!("# {}", self.get_state());
                let message = XBoardOut::Custom(state);
                self.comm.send(EngineOutput::XBoard(message));
            }
            XBoardIn::DebugOn => self.debug = true,
            XBoardIn::DebugOff => self.debug = false,
            XBoardIn::Board => {
                let print_board = format!("{}", self.board.lock().expect(ErrFatal::LOCK));
                let message = XBoardOut::Custom(print_board);
                self.comm.send(EngineOutput::XBoard(message));
            }
            XBoardIn::Ignore(cmd) => {
                if self.debug {
                    let ignored = format!("Notification \"{cmd}\" is ignored.");
                    let message = XBoardOut::Custom(ignored);
                    self.comm.send(EngineOutput::XBoard(message));
                }
            }
            XBoardIn::Unknown(cmd) => {
                if self.debug {
                    let error = String::from(ErrXboard::UNKNOWN_CMD);
                    let message = XBoardOut::Error(error, cmd);
                    self.comm.send(EngineOutput::XBoard(message));
                }
            }
        }
    }

    fn inapplicable_command(cmd: XBoardIn) -> XBoardOut {
        let error = ErrXboard::INAPPLICABLE_CMD.to_string();
        let cmd = cmd.to_string().to_lowercase();

        XBoardOut::Error(error, cmd)
    }
}
