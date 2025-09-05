use crate::{
    basetypes::error::ErrFatal,
    communication::{
        defs::EngineOutput,
        feature::Feature,
        uci::{Uci, cmd_out::UciOut, print},
    },
};
use std::{
    sync::{Arc, mpsc::channel},
    thread,
};

impl Uci {
    // The UCI output thread receives information from the engine thread.
    pub fn output_thread(&mut self, features: Arc<Vec<Feature>>) {
        // Create an incoming channel where the information is received.
        let (tx_by_engine, rx_from_engine) = channel::<EngineOutput>();

        // Clone this because we can't capture variables internal to the
        // struct. We only need this only once as a reply to the "id"
        // command when the engine has started.
        let about = self.about.clone();

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
                            print::id(about.get_engine(), about.get_version(), about.get_author());
                            print::features(&features);
                            print::uciok();
                        }
                        UciOut::ReadyOk => print::readyok(),
                        UciOut::InfoString(message) => print::info_string(&message),
                        UciOut::Custom(message) => print::custom(&message),
                        UciOut::SearchSummary(summary) => print::search_summary(&summary),
                        UciOut::SearchCurrMove(current) => print::search_currmove(&current),
                        UciOut::SearchStats(stats) => print::search_stats(&stats),
                        UciOut::BestMove(bestmove) => print::best_move(&bestmove),
                        UciOut::Quit => break, // This will shut down the input thread.
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
