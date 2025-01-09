use crate::engine::Engine;
use librustic::communication::uci::{cmd_in::UciIn, cmd_out::UciOut};

const UNKNOWN: &str = "Unknown command";

// This block implements handling of incoming information, which will be in
// the form of either Comm or Search reports.
impl Engine {
    pub fn comm_handler(&mut self, input: UciIn) {
        match input {
            UciIn::Quit => self.quit(),
            UciIn::Uci => self.comm.send(UciOut::Id),
            UciIn::IsReady => self.comm.send(UciOut::ReadyOk),
            UciIn::Unknown(cmd) => self
                .comm
                .send(UciOut::InfoString(format!("{UNKNOWN}: {cmd}"))),
            // CommIn::Uci(command) => self.uci_handler(command),
            // CommIn::XBoard(command) => self.xboard_handler(command),

            // CommIn::Quit => self.quit(),
            // CommIn::Board => self.comm.send(CommOut::PrintBoard),
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
