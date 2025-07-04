use crate::engine::Engine;
use librustic::{
    basetypes::error::{ErrFatal, ErrNormal, ErrUci},
    communication::{
        defs::EngineOutput,
        uci::{
            cmd_in::UciIn,
            cmd_out::UciOut,
            defs::{Name, Value},
        },
    },
    defs::FEN_START_POSITION,
    search::defs::{SearchControl, SearchMode, SearchParams, SAFEGUARD},
};

// This is the UCI handler. It handles the incoming UCI commands and when
// needed, it sends replies to the communication module to be sent out of
// the engine to the GUI. Each enum variant of the UciIn type has a
// match-arm in this function.
impl Engine {
    pub fn uci_handler(&mut self, command: UciIn) {
        let mut search_params = SearchParams::new();
        search_params.verbosity = self.settings.verbosity;

        match command {
            UciIn::Uci => self.comm.send(EngineOutput::Uci(UciOut::Id)),
            UciIn::IsReady => self.comm.send(EngineOutput::Uci(UciOut::ReadyOk)),
            UciIn::UciNewGame => {
                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_setup(Some(FEN_START_POSITION))
                    .expect(ErrFatal::NEW_GAME);
                self.search.transposition_clear();
            }
            UciIn::DebugOn => self.debug = true,
            UciIn::DebugOff => self.debug = false,
            UciIn::Stop => self.search.send(SearchControl::Stop),
            UciIn::Quit => self.quit(),
            UciIn::Position(fen, moves) => {
                let fen_result = self
                    .board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_setup(Some(fen.as_str()));

                if fen_result.is_ok() {
                    for m in moves.iter() {
                        if !self.execute_move(m) && self.debug {
                            let fail = format!("{}: {}", ErrNormal::NOT_LEGAL, m.clone());
                            self.comm.send(EngineOutput::Uci(UciOut::InfoString(fail)));
                            break;
                        }
                    }
                } else if self.debug {
                    let fail = format!("{}: {}", ErrNormal::FEN_FAILED, fen.clone());
                    self.comm.send(EngineOutput::Uci(UciOut::InfoString(fail)));
                }
            }
            UciIn::GoInfinite => {
                search_params.search_mode = SearchMode::Infinite;
                self.search.send(SearchControl::Start(search_params));
            }
            UciIn::GoDepth(depth) => {
                search_params.depth = depth;
                search_params.search_mode = SearchMode::Depth;
                self.search.send(SearchControl::Start(search_params));
            }
            UciIn::GoMoveTime(msecs) => {
                search_params.move_time = msecs - (SAFEGUARD as u128);
                search_params.search_mode = SearchMode::MoveTime;
                self.search.send(SearchControl::Start(search_params));
            }
            UciIn::GoNodes(nodes) => {
                search_params.nodes = nodes;
                search_params.search_mode = SearchMode::Nodes;
                self.search.send(SearchControl::Start(search_params));
            }
            UciIn::GoGameTime(gt) => {
                search_params.game_time = gt;
                search_params.search_mode = SearchMode::GameTime;
                self.search.send(SearchControl::Start(search_params));
            }
            UciIn::SetOption(name, value) => {
                if !name.is_empty() {
                    self.setoption(name, value);
                } else if self.debug {
                    let message = UciOut::InfoString(String::from(ErrUci::OPTION_NO_NAME));
                    self.comm.send(EngineOutput::Uci(message));
                }
            }
            UciIn::Unknown(cmd) => self.uci_unknown(cmd),
        }
    }

    // UCI calls an engine Feature an "Option". This function handles
    // setting the engine's features/options depending on the incoming
    // option names and values.
    fn setoption(&mut self, option: Name, value: Value) {
        match option {
            option if option == "hash" => self.setoption_hash(option, value),
            option if option == "clear hash" => {
                self.search.transposition_clear();
            }
            _ => {
                if self.debug {
                    let message =
                        UciOut::InfoString(format!("{}: {}", ErrUci::OPTION_UNKNOWN_NAME, option));
                    self.comm.send(EngineOutput::Uci(message));
                }
            }
        }
    }

    // Setting the Hash feature requires error checking for the incoming
    // value so this has been extracted into its own function.
    fn setoption_hash(&mut self, option: Name, value: Value) {
        if value.is_none() && self.debug {
            let message = UciOut::InfoString(format!("{}: {}", ErrUci::OPTION_NO_VALUE, option));
            self.comm.send(EngineOutput::Uci(message));
            return;
        }

        if let Some(value) = value {
            if let Ok(size) = value.parse::<usize>() {
                self.settings.tt_size = size;
                self.search.transposition_resize(size);
            } else if self.debug {
                let message =
                    UciOut::InfoString(format!("{}: {}", ErrUci::OPTION_VALUE_NOT_INT, option));
                self.comm.send(EngineOutput::Uci(message));
            }
        }
    }

    // This function handles commands that cannot be captured in one of the
    // UciIn enum variants; these are therefore unknown. These could be any
    // string of text. The engine can do whatever it wants with these; most
    //of the time they are ignored (except in debug mode, where an
    //InfoString is printed.) In Rustic, the function handles "board" as a
    // custom command that is not part of the UCI-specification. It may
    // handle other incoming commands as custom in the future as well.
    fn uci_unknown(&self, cmd: String) {
        match cmd {
            c if c == "board" => {
                let board = format!("{}", self.board.lock().expect(ErrFatal::LOCK));
                self.comm.send(EngineOutput::Uci(UciOut::Custom(board)));
            }
            _ => {
                if self.debug {
                    let message = UciOut::InfoString(format!("{}: {}", ErrUci::UNKNOWN_CMD, cmd));
                    self.comm.send(EngineOutput::Uci(message));
                }
            }
        }
    }
}
