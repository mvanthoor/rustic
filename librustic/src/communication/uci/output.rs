use crate::{
    basetypes::error::ErrFatal,
    board::Board,
    communication::{
        shared,
        uci::uci_option::{UciOption, UiElement},
    },
    movegen::defs::Move,
    search::defs::{SearchCurrentMove, SearchStats, SearchSummary},
};
use std::sync::{Arc, Mutex};

pub fn id(engine: &str, version: &str, author: &str) {
    println!("id name {} {}", engine, version);
    println!("id author {}", author);
}

pub fn readyok() {
    println!("readyok");
}

pub fn info_string(message: &String) {
    println!("info string {message}");
}

pub fn features(features: &Arc<Vec<UciOption>>) {
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

        let uci_feature = format!("{name} {ui_element} {value_default} {value_min} {value_max}")
            .trim()
            .to_string();

        println!("{uci_feature}");
    }
}

pub fn uciok() {
    println!("uciok");
}

pub fn search_summary(s: &SearchSummary) {
    // Report depth and seldepth (if available).
    let depth = if s.seldepth > 0 {
        format!("depth {} seldepth {}", s.depth, s.seldepth)
    } else {
        format!("depth {}", s.depth)
    };

    // If mate found, report this; otherwise report normal score.
    let score = if let Some(moves) = shared::moves_to_checkmate(s.cp) {
        // If the engine is being mated itself, flip the score.
        let flip = if s.cp < 0 { -1 } else { 1 };
        format!("mate {}", moves * flip)
    } else {
        format!("cp {}", s.cp)
    };

    // Only display hash full if not 0
    let hash_full = if s.hash_full > 0 {
        format!(" hashfull {} ", s.hash_full)
    } else {
        String::from(" ")
    };

    let info = format!(
        "info {} score {} time {} nodes {} nps {}{}pv {}",
        depth,
        score,
        s.time,
        s.nodes,
        s.nps,
        hash_full,
        s.pv_to_string(),
    );

    println!("{info}");
}

pub fn search_currmove(c: &SearchCurrentMove) {
    println!(
        "info currmove {} currmovenumber {}",
        c.curr_move, c.curr_move_number
    );
}

pub fn search_stats(s: &SearchStats) {
    let hash_full = if s.hash_full > 0 {
        format!(" hashfull {}", s.hash_full)
    } else {
        String::from("")
    };

    println!(
        "info time {} nodes {} nps {}{}",
        s.time, s.nodes, s.nps, hash_full
    );
}

pub fn best_move(m: &Move) {
    println!("bestmove {m}");
}

pub fn print_board(board: &Arc<Mutex<Board>>) {
    println!("{}", &board.lock().expect(ErrFatal::LOCK));
}
