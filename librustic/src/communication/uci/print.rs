use crate::{
    communication::{
        feature::{Feature, IFeature, UiElement},
        shared,
    },
    movegen::defs::Move,
    search::defs::{SearchCurrentMove, SearchStats, SearchSummary},
};
use std::sync::Arc;

pub fn id(engine: &str, version: &str, author: &str) {
    println!("id name {} {}", engine, version);
    println!("id author {}", author);
}

pub fn readyok() {
    println!("readyok");
}

pub fn info_string(message: &str) {
    println!("info string {message}");
}

pub fn features(features: &Arc<Vec<Feature>>) {
    for feature in features.iter() {
        let name = format!("option name {}", feature.get_name());

        let ui_element = if let Some(e) = feature.get_ui_element() {
            match e {
                UiElement::Spin => String::from("type spin"),
                UiElement::Button => String::from("type button"),
            }
        } else {
            String::from("")
        };

        let value_default = if let Some(v) = feature.get_default() {
            format!("default {}", v)
        } else {
            String::from("")
        };

        let value_min = if let Some(v) = feature.get_min() {
            format!("min {}", v)
        } else {
            String::from("")
        };

        let value_max = if let Some(v) = feature.get_max() {
            format!("max {}", v)
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

pub fn custom(message: &str) {
    println!("{message}");
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
