use crate::{
    basetypes::error::ErrFatal,
    communication::{
        defs::EngineOutput,
        feature::Feature,
        uci::{cmd_out::UciOut, output, Uci},
    },
};
use std::{
    sync::{mpsc::channel, Arc},
    thread,
};

impl Uci {
    // The control thread receives commands from the engine thread.
    pub fn output_thread(&mut self, features: Arc<Vec<Feature>>) {
        // Create an incoming channel for the output thread.
        let (transmitter_for_engine, received_from_engine) = channel::<EngineOutput>();
        let about = self.about.clone();

        // Create the output thread.
        let thread = thread::spawn(move || {
            loop {
                let print_to_stdio = received_from_engine.recv().expect(ErrFatal::CHANNEL);

                // Perform command as sent by the engine thread.
                if let EngineOutput::Uci(cmd) = print_to_stdio {
                    match cmd {
                        UciOut::Id => {
                            output::id(about.get_engine(), about.get_version(), about.get_author());
                            output::features(&features);
                            output::uciok();
                        }
                        UciOut::ReadyOk => output::readyok(),
                        UciOut::InfoString(message) => output::info_string(&message),
                        UciOut::Quit => break,
                        UciOut::Custom(message) => output::custom(&message),
                        UciOut::SearchSummary(summary) => output::search_summary(&summary),
                        UciOut::SearchCurrMove(current) => output::search_currmove(&current),
                        UciOut::SearchStats(stats) => output::search_stats(&stats),
                        UciOut::BestMove(bestmove) => output::best_move(&bestmove),
                    }
                }
            }
        });

        // Store handle and control sender.
        self.output_thread = Some(thread);
        self.output_write = Some(transmitter_for_engine);
    }
}
