use crate::{
    basetypes::error::ErrFatal,
    communication::{
        defs::EngineOutput,
        uci::{Uci, cmd_out::UciOut, print},
    },
};
use std::{sync::mpsc::channel, thread};

impl Uci {
    // The UCI output thread receives information from the engine thread.
    pub fn thread_out(&mut self) {
        // Create an incoming channel where the information is received.
        let (tx_by_engine, rx_from_engine) = channel::<EngineOutput>();
        let engine_info = self.engine_info.clone();
        let features = self.features.clone();

        // Create the output thread. This will print information sent by
        // the engine thread to stdio. The information will either end up
        // on screen, or be captured by a GUI depending on how the engine
        // is being run.
        let thread = thread::spawn(move || {
            loop {
                // The receiver will block the thread until information is received.
                let print_to_stdio = rx_from_engine.recv().expect(ErrFatal::CHANNEL);

                // EngineOutput could also contain output destined for
                // other protocols than UCI. We are only interested in the
                // UCI-part in this thread.
                if let EngineOutput::Uci(cmd) = print_to_stdio {
                    match cmd {
                        UciOut::Id => {
                            print::id(&engine_info);
                            print::features(&features);
                            print::uciok();
                        }
                        UciOut::ReadyOk => print::readyok(),
                        UciOut::InfoString(message) => print::info_string(&message),
                        UciOut::SearchSummary(summary) => print::search_summary(&summary),
                        UciOut::SearchCurrMove(current) => print::search_currmove(&current),
                        UciOut::SearchStats(stats) => print::search_stats(&stats),
                        UciOut::BestMove(bestmove) => print::best_move(&bestmove),
                        UciOut::Quit => break,
                        UciOut::Custom(msg) => print::custom(msg),
                    }
                }
            }
        });

        // Store the thread's handle and output transmitter for use by the
        // engine thread.
        self.output_thread = Some(thread);
        self.output_write = Some(tx_by_engine);
    }
}
