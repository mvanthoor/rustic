use super::{
    defs::{ErrFatal, ErrNormal},
    Engine,
};
use crate::comm::{uci::UciReport, CommControl, CommReport};

// This block implements handling of incoming information, which will be in
// the form of either Comm or Search reports.
impl Engine {
    pub fn comm_reports(&mut self, comm_report: &CommReport) {
        // Split out the comm reports according to their source.
        match comm_report {
            CommReport::Uci(u) => self.comm_reports_uci(u),
        }
    }

    // Handles "Uci" Comm reports sent by the UCI-module.
    fn comm_reports_uci(&mut self, u: &UciReport) {
        match u {
            // Uci commands
            UciReport::Uci => self.comm.send(CommControl::Identify),
            UciReport::IsReady => self.comm.send(CommControl::Ready),
            UciReport::Position(fen, moves) => {
                let fen_result = self.board.lock().expect(ErrFatal::LOCK).fen_read(Some(fen));

                if fen_result.is_ok() {
                    for m in moves.iter() {
                        let ok = self.execute_move(m.clone());
                        if !ok {
                            let msg = format!("{}: {}", m, ErrNormal::NOT_LEGAL);
                            self.comm.send(CommControl::PrintMessage(msg));
                            break;
                        }
                    }
                }

                if fen_result.is_err() {
                    let msg = ErrNormal::FEN_FAILED.to_string();
                    self.comm.send(CommControl::PrintMessage(msg));
                }
            }

            // Quit received. Shut down engine.
            UciReport::Quit => self.quit(),

            // Custom commands
            UciReport::Board => self.comm.send(CommControl::PrintBoard),
            UciReport::Help => self.comm.send(CommControl::PrintHelp),
            UciReport::Unknown => (),
        }
    }
}
