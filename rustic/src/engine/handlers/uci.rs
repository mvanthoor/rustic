use crate::engine::Engine;
use librustic::{
    basetypes::error::{ErrFatal, ErrNormal},
    communication::uci::{cmd_in::UciIn, cmd_out::UciOut},
    defs::FEN_START_POSITION,
};

const UNKNOWN: &str = "Unknown command";

// This block implements handling of incoming information, which will be in
// the form of either Comm or Search reports.
impl Engine {
    pub fn comm_handler(&mut self, input: UciIn) {
        match input {
            UciIn::Quit => self.quit(),
            UciIn::Uci => self.comm.send(UciOut::Id),
            UciIn::IsReady => self.comm.send(UciOut::ReadyOk),
            UciIn::UciNewGame => {
                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_setup(Some(FEN_START_POSITION))
                    .expect(ErrFatal::NEW_GAME);
                self.search.transposition_clear();
            }
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
                            self.comm.send(UciOut::InfoString(fail));
                            break;
                        }
                    }
                } else if self.debug {
                    let fail = format!("{}: {}", ErrNormal::FEN_FAILED, fen.clone());
                    self.comm.send(UciOut::InfoString(fail));
                }
            }
            UciIn::DebugOn => self.debug = true,
            UciIn::DebugOff => self.debug = false,
            UciIn::Unknown(cmd) => {
                if self.debug {
                    self.comm
                        .send(UciOut::InfoString(format!("{UNKNOWN}: {cmd}")));
                }
            }
            UciIn::Board => self.comm.send(UciOut::PrintBoard),
            // CommIn::History => self.comm.send(CommOut::PrintHistory),
            // CommIn::Eval => {
            //     let mtx_board = &self.board.lock().expect(ErrFatal::LOCK);
            //     let eval = Evaluation::evaluate_position(mtx_board);
            //     let phase = mtx_board.game_state.phase_value;
            //     self.comm.send(CommOut::PrintEval(eval, phase));
            // }
            // CommIn::State => self.comm.send(CommOut::PrintState(self.state)),
            // CommIn::ClearTt => {
            //     self.search.transposition_clear();
            //     self.comm
            //         .send(CommOut::Message(Messages::CLEARED_TT.to_string()));
            // }
            // CommIn::Help => self.comm.send(CommOut::PrintHelp),
            // CommIn::Ignore(cmd) => {
            //     self.comm.send(CommOut::Message(format!(
            //         "{}: {}",
            //         Messages::COMMAND_IGNORED,
            //         cmd
            //     )));
            // }
            // CommIn::Unknown(cmd) => self
            //     .comm
            //     .send(CommOut::Error(ErrNormal::UNKNOWN_COMMAND, cmd.to_string())),
        }
    }
}
