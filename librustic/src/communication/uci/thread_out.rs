use crate::{
    basetypes::error::ErrFatal,
    board::Board,
    communication::{
        feature::{Feature, UiElement},
        uci::{cmd_out::UciOut, Uci},
    },
};
use std::{
    sync::{mpsc::channel, Arc, Mutex},
    thread,
};

impl Uci {
    // The control thread receives commands from the engine thread.
    pub fn output_thread(&mut self, _board: Arc<Mutex<Board>>, features: Arc<Vec<Feature>>) {
        // Create an incoming channel for the output thread.
        let (transmitter_for_engine, received_from_engine) = channel();
        let about = self.about.clone();

        // let t_board = Arc::clone(&board);

        // Create the output thread.
        let thread = thread::spawn(move || {
            loop {
                let print_to_stdio = received_from_engine.recv().expect(ErrFatal::CHANNEL);

                // Perform command as sent by the engine thread.
                match print_to_stdio {
                    UciOut::Id => {
                        Uci::id(about.get_engine(), about.get_version(), about.get_author());
                        Uci::features(&features);
                        Uci::uciok();
                    }
                    UciOut::Quit => break,
                    // CommOut::Uci(UciOut::Ready) => Uci::readyok(),
                    // CommOut::Message(msg) => Uci::message(&msg),
                    // CommOut::SearchSummary(summary) => Uci::search_summary(&summary),
                    // CommOut::SearchCurrMove(current) => Uci::search_currmove(&current),
                    // CommOut::SearchStats(stats) => Uci::search_stats(&stats),
                    // CommOut::BestMove(bm, result) => Uci::best_move(&bm, &result),
                    // CommOut::Error(err_type, cmd) => Uci::error(err_type, &cmd),

                    // // Custom commands
                    // CommOut::PrintBoard => Shared::print_board(&t_board),
                    // CommOut::PrintHistory => Shared::print_history(&t_board),
                    // CommOut::PrintEval(eval, phase) => Shared::print_eval(eval, phase),
                    // CommOut::PrintState(state) => Shared::print_state(&state),
                    // CommOut::PrintHelp => Shared::print_help(CommType::UCI),

                    // Ignore everything else
                    // _ => (),
                }
            }
        });

        // Store handle and control sender.
        self.output_thread = Some(thread);
        self.uci_output = Some(transmitter_for_engine);
    }
}

impl Uci {
    fn id(engine: &str, version: &str, author: &str) {
        println!("id name {} {}", engine, version);
        println!("id author {}", author);
    }

    fn features(features: &Arc<Vec<Feature>>) {
        for feature in features.iter() {
            let name = format!("option name {}", feature.name);

            let ui_element = match feature.ui_element {
                UiElement::Spin => String::from("type spin"),
                UiElement::Button => String::from("type button"),
            };

            let value_default = if let Some(v) = &feature.default {
                format!("default {}", (*v).clone())
            } else {
                String::from("")
            };

            let value_min = if let Some(v) = &feature.min {
                format!("min {}", (*v).clone())
            } else {
                String::from("")
            };

            let value_max = if let Some(v) = &feature.max {
                format!("max {}", (*v).clone())
            } else {
                String::from("")
            };

            let uci_feature =
                format!("{name} {ui_element} {value_default} {value_min} {value_max}")
                    .trim()
                    .to_string();

            println!("{uci_feature}");
        }
    }

    fn uciok() {
        println!("uciok");
    }
}
