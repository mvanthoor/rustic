/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

// This file implements the UCI communication module.

use super::{CommControl, CommReport, CommType, IComm};
use crate::{
    board::Board,
    defs::{About, FEN_START_POSITION},
    engine::defs::{EngineOption, EngineOptionName, ErrFatal, Information, UiElement},
    misc::print,
    movegen::defs::Move,
    search::defs::{
        GameTime, SearchCurrentMove, SearchStats, SearchSummary, CHECKMATE, CHECKMATE_THRESHOLD,
    },
};
use crossbeam_channel::{self, Sender};
use std::{
    io::{self},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

// Input will be turned into a report, which wil be sent to the engine. The
// main engine thread will react accordingly.
#[derive(PartialEq, Clone)]
pub enum UciReport {
    Uci,
    UciNewGame,
    IsReady,
    SetOption(EngineOptionName),
    Position(String, Vec<String>),
    GoInfinite,
    GoDepth(i8),
    GoMoveTime(u128),
    GoNodes(usize),
    GoGameTime(GameTime),
    Stop,
    Quit,

    // Custom
    Board,
    History,
    Eval,
    Help,

    // Empty or unknown command.
    Unknown,
}

// This struct is used to instantiate the Comm Console module.
pub struct Uci {
    control_handle: Option<JoinHandle<()>>,
    report_handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<CommControl>>,
}

// Public functions
impl Uci {
    // Create a new console.
    pub fn new() -> Self {
        Self {
            control_handle: None,
            report_handle: None,
            control_tx: None,
        }
    }
}

// Any communication module must implement the trait IComm.
impl IComm for Uci {
    fn init(
        &mut self,
        report_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<EngineOption>>,
    ) {
        // Start threads
        self.report_thread(report_tx);
        self.control_thread(board, options);
    }

    // The creator of the Comm module can use this function to send
    // messages or commands into the Control thread.
    fn send(&self, msg: CommControl) {
        if let Some(tx) = &self.control_tx {
            tx.send(msg).expect(ErrFatal::CHANNEL);
        }
    }

    // After the engine sends 'quit' to the control thread, it will call
    // wait_for_shutdown() and then wait here until shutdown is completed.
    fn wait_for_shutdown(&mut self) {
        if let Some(h) = self.report_handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }

        if let Some(h) = self.control_handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }

    // This function just returns the name of the communication protocol.
    fn get_protocol_name(&self) -> &'static str {
        CommType::UCI
    }
}

// Implement the report thr
impl Uci {
    // The Report thread sends incoming data to the engine thread.
    fn report_thread(&mut self, report_tx: Sender<Information>) {
        // Create thread-local variables
        let mut t_incoming_data = String::from("");
        let t_report_tx = report_tx; // Report sender

        // Actual thread creation.
        let report_handle = thread::spawn(move || {
            let mut quit = false;

            // Keep running as long as 'quit' is not detected.
            while !quit {
                // Get data from stdin.
                io::stdin()
                    .read_line(&mut t_incoming_data)
                    .expect(ErrFatal::READ_IO);

                // Create a report from the incoming data.
                let new_report = Uci::create_report(&t_incoming_data);

                // Check if the created report is valid, so it is something
                // the engine will understand.
                if new_report.is_valid() {
                    // Send it to the engine thread.
                    t_report_tx
                        .send(Information::Comm(new_report.clone()))
                        .expect(ErrFatal::HANDLE);

                    // Terminate the reporting thread if "Quit" was detected.
                    quit = new_report == CommReport::Uci(UciReport::Quit);
                }

                // Clear for next input
                t_incoming_data = String::from("");
            }
        });

        // Store the handle.
        self.report_handle = Some(report_handle);
    }
}

// Implement the control thread
impl Uci {
    // The control thread receives commands from the engine thread.
    fn control_thread(&mut self, board: Arc<Mutex<Board>>, options: Arc<Vec<EngineOption>>) {
        // Create an incoming channel for the control thread.
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<CommControl>();

        // Create the control thread.
        let control_handle = thread::spawn(move || {
            let mut quit = false;
            let t_board = Arc::clone(&board);
            let t_options = Arc::clone(&options);

            // Keep running as long as Quit is not received.
            while !quit {
                let control = control_rx.recv().expect(ErrFatal::CHANNEL);

                // Perform command as sent by the engine thread.
                match control {
                    CommControl::Identify => {
                        Uci::id();
                        Uci::options(&t_options);
                        Uci::uciok();
                    }
                    CommControl::Ready => Uci::readyok(),
                    CommControl::Quit => quit = true,
                    CommControl::SearchSummary(summary) => Uci::search_summary(&summary),
                    CommControl::SearchCurrMove(current) => Uci::search_currmove(&current),
                    CommControl::SearchStats(stats) => Uci::search_stats(&stats),
                    CommControl::InfoString(msg) => Uci::info_string(&msg),
                    CommControl::BestMove(bm) => Uci::best_move(&bm),

                    // Custom prints for use in the console.
                    CommControl::PrintBoard => Uci::print_board(&t_board),
                    CommControl::PrintHistory => Uci::print_history(&t_board),
                    CommControl::PrintHelp => Uci::print_help(),

                    // Comm Control commands that are not (yet) used.
                    CommControl::Update => (),
                }
            }
        });

        // Store handle and control sender.
        self.control_handle = Some(control_handle);
        self.control_tx = Some(control_tx);
    }
}

// Private functions for this module.
impl Uci {
    // This function turns the incoming data into UciReports which the
    // engine is able to understand and react to.
    fn create_report(input: &str) -> CommReport {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match i {
            // UCI commands
            cmd if cmd == "uci" => CommReport::Uci(UciReport::Uci),
            cmd if cmd == "ucinewgame" => CommReport::Uci(UciReport::UciNewGame),
            cmd if cmd == "isready" => CommReport::Uci(UciReport::IsReady),
            cmd if cmd == "stop" => CommReport::Uci(UciReport::Stop),
            cmd if cmd.starts_with("setoption") => Uci::parse_setoption(&cmd),
            cmd if cmd.starts_with("position") => Uci::parse_position(&cmd),
            cmd if cmd.starts_with("go") => Uci::parse_go(&cmd),
            cmd if cmd == "quit" || cmd == "exit" || cmd.is_empty() => {
                CommReport::Uci(UciReport::Quit)
            }

            // Custom commands
            cmd if cmd == "board" => CommReport::Uci(UciReport::Board),
            cmd if cmd == "history" => CommReport::Uci(UciReport::History),
            cmd if cmd == "eval" => CommReport::Uci(UciReport::Eval),
            cmd if cmd == "help" => CommReport::Uci(UciReport::Help),

            // Everything else is ignored.
            _ => CommReport::Uci(UciReport::Unknown),
        }
    }

    fn parse_position(cmd: &str) -> CommReport {
        enum Tokens {
            Nothing,
            Fen,
            Moves,
        }

        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
        let mut fen = String::from("");
        let mut moves: Vec<String> = Vec::new();
        let mut skip_fen = false;
        let mut token = Tokens::Nothing;

        for p in parts {
            match p {
                t if t == "position" => (), // Skip. We know we're parsing "position".
                t if t == "startpos" => skip_fen = true, // "fen" is now invalidated.
                t if t == "fen" && !skip_fen => token = Tokens::Fen,
                t if t == "moves" => token = Tokens::Moves,
                _ => match token {
                    Tokens::Nothing => (),
                    Tokens::Fen => {
                        fen.push_str(&p[..]);
                        fen.push(' ');
                    }
                    Tokens::Moves => moves.push(p),
                },
            }
        }
        // No FEN part in the command. Use the start position.
        if fen.is_empty() {
            fen = String::from(FEN_START_POSITION)
        }
        CommReport::Uci(UciReport::Position(fen.trim().to_string(), moves))
    }

    fn parse_go(cmd: &str) -> CommReport {
        enum Tokens {
            Nothing,
            Depth,
            Nodes,
            MoveTime,
            WTime,
            BTime,
            WInc,
            BInc,
            MovesToGo,
        }

        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
        let mut report = CommReport::Uci(UciReport::Unknown);
        let mut token = Tokens::Nothing;
        let mut game_time = GameTime::new(0, 0, 0, 0, None);

        for p in parts {
            match p {
                t if t == "go" => report = CommReport::Uci(UciReport::GoInfinite),
                t if t == "infinite" => break, // Already Infinite; nothing more to do.
                t if t == "depth" => token = Tokens::Depth,
                t if t == "movetime" => token = Tokens::MoveTime,
                t if t == "nodes" => token = Tokens::Nodes,
                t if t == "wtime" => token = Tokens::WTime,
                t if t == "btime" => token = Tokens::BTime,
                t if t == "winc" => token = Tokens::WInc,
                t if t == "binc" => token = Tokens::BInc,
                t if t == "movestogo" => token = Tokens::MovesToGo,
                _ => match token {
                    Tokens::Nothing => (),
                    Tokens::Depth => {
                        let depth = p.parse::<i8>().unwrap_or(1);
                        report = CommReport::Uci(UciReport::GoDepth(depth));
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::MoveTime => {
                        let milliseconds = p.parse::<u128>().unwrap_or(1000);
                        report = CommReport::Uci(UciReport::GoMoveTime(milliseconds));
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::Nodes => {
                        let nodes = p.parse::<usize>().unwrap_or(1);
                        report = CommReport::Uci(UciReport::GoNodes(nodes));
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::WTime => game_time.wtime = p.parse::<u128>().unwrap_or(0),
                    Tokens::BTime => game_time.btime = p.parse::<u128>().unwrap_or(0),
                    Tokens::WInc => game_time.winc = p.parse::<u128>().unwrap_or(0),
                    Tokens::BInc => game_time.binc = p.parse::<u128>().unwrap_or(0),
                    Tokens::MovesToGo => {
                        game_time.moves_to_go = if let Ok(x) = p.parse::<usize>() {
                            Some(x)
                        } else {
                            None
                        }
                    }
                }, // end match token
            } // end match p
        } // end for

        // If we are still in the default "go infinite" mode, we must
        // switch to GameTime mode if at least one parameter of "go wtime
        // btime winc binc" was set to something else but 0.
        let is_default_mode = report == CommReport::Uci(UciReport::GoInfinite);
        let has_time = game_time.wtime > 0 || game_time.btime > 0;
        let has_inc = game_time.winc > 0 || game_time.binc > 0;
        let is_game_time = has_time || has_inc;
        if is_default_mode && is_game_time {
            report = CommReport::Uci(UciReport::GoGameTime(game_time));
        }

        report
    } // end parse_go()

    fn parse_setoption(cmd: &str) -> CommReport {
        enum Tokens {
            Nothing,
            Name,
            Value,
        }

        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
        let mut token = Tokens::Nothing;
        let mut name = String::from(""); // Option name provided by the UCI command.
        let mut value = String::from(""); // Option value provided by the UCI command.
        let mut eon = EngineOptionName::Nothing; // Engine Option Name to send to the engine.

        for p in parts {
            match p {
                t if t == "setoption" => (),
                t if t == "name" => token = Tokens::Name,
                t if t == "value" => token = Tokens::Value,
                _ => match token {
                    Tokens::Name => name = format!("{} {}", name, p),
                    Tokens::Value => value = p.to_lowercase(),
                    Tokens::Nothing => (),
                },
            }
        }

        // Determine which engine option name to send.
        if !name.is_empty() {
            name = name.to_lowercase().trim().to_string();
            match &name[..] {
                "hash" => eon = EngineOptionName::Hash(value),
                "clear hash" => eon = EngineOptionName::ClearHash,
                _ => (),
            }
        }

        // Send the engine option name with value to the engine thread.
        CommReport::Uci(UciReport::SetOption(eon))
    }
}

// Implements UCI responses to send to the G(UI).
impl Uci {
    fn id() {
        println!("id name {} {}", About::ENGINE, About::VERSION);
        println!("id author {}", About::AUTHOR);
    }

    fn options(options: &Arc<Vec<EngineOption>>) {
        for o in options.iter() {
            let name = format!("option name {}", o.name);

            let ui_element = match o.ui_element {
                UiElement::Spin => String::from("type spin"),
                UiElement::Button => String::from("type button"),
            };

            let value_default = if let Some(v) = &o.default {
                format!("default {}", (*v).clone())
            } else {
                String::from("")
            };

            let value_min = if let Some(v) = &o.min {
                format!("min {}", (*v).clone())
            } else {
                String::from("")
            };

            let value_max = if let Some(v) = &o.max {
                format!("max {}", (*v).clone())
            } else {
                String::from("")
            };

            let option = format!(
                "{} {} {} {} {}",
                name, ui_element, value_default, value_min, value_max
            )
            .trim()
            .to_string();

            println!("{}", option);
        }
    }

    fn uciok() {
        println!("uciok");
    }

    fn readyok() {
        println!("readyok");
    }

    fn search_summary(s: &SearchSummary) {
        // If mate found, report this; otherwise report normal score.
        let score = if (s.cp.abs() >= CHECKMATE_THRESHOLD) && (s.cp.abs() < CHECKMATE) {
            // Number of plies to mate.
            let ply = CHECKMATE - s.cp.abs();

            // Check if the number of ply's is odd
            let is_odd = ply % 2 == 1;

            // Calculate number of moves to mate
            let moves = if is_odd { (ply + 1) / 2 } else { ply / 2 };

            // If the engine is being mated itself, flip the score.
            let flip = if s.cp < 0 { -1 } else { 1 };

            // Report the mate
            format!("mate {}", moves * flip)
        } else {
            // Report the normal score if there's no mate detected.
            format!("cp {}", s.cp)
        };

        // Report depth and seldepth (if available).
        let depth = if s.seldepth > 0 {
            format!("depth {} seldepth {}", s.depth, s.seldepth)
        } else {
            format!("depth {}", s.depth)
        };

        // Only display hash full if not 0
        let hash_full = if s.hash_full > 0 {
            format!(" hashfull {} ", s.hash_full)
        } else {
            String::from(" ")
        };

        let pv = s.pv_as_string();

        let info = format!(
            "info score {} {} time {} nodes {} nps {}{}pv {}",
            score, depth, s.time, s.nodes, s.nps, hash_full, pv,
        );

        println!("{}", info);
    }

    fn search_currmove(c: &SearchCurrentMove) {
        println!(
            "info currmove {} currmovenumber {}",
            c.curr_move.as_string(),
            c.curr_move_number
        );
    }

    fn search_stats(s: &SearchStats) {
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

    fn info_string(msg: &str) {
        println!("info string {}", msg);
    }

    fn best_move(m: &Move) {
        println!("bestmove {}", m.as_string());
    }
}

// implements handling of custom commands. These are mostly used when using
// the UCI protocol directly in a terminal window.
impl Uci {
    fn print_board(board: &Arc<Mutex<Board>>) {
        print::position(&board.lock().expect(ErrFatal::LOCK), None);
    }

    fn print_history(board: &Arc<Mutex<Board>>) {
        let mtx_board = board.lock().expect(ErrFatal::LOCK);
        let length = mtx_board.history.len();

        if length == 0 {
            println!("No history available.");
        }

        for i in 0..length {
            let h = mtx_board.history.get_ref(i);
            println!("{:<3}| ply: {} {}", i, i + 1, h.as_string());
        }

        std::mem::drop(mtx_board);
    }

    fn print_help() {
        println!("The engine is in UCI communication mode. It supports some custom");
        println!("non-UCI commands to make use through a terminal window easier.");
        println!("These commands can also be very useful for debugging purposes.");
        println!();
        println!("Custom commands");
        println!("================================================================");
        println!("help      :   This help information.");
        println!("board     :   Print the current board state.");
        println!("history   :   Print a list of past board states.");
        println!("eval      :   Print evaluation for side to move.");
        println!("exit      :   Quit/Exit the engine.");
        println!();
    }
}
