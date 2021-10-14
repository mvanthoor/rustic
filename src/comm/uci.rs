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

use super::{shared::Shared, CommInput, CommOutput, CommType, IComm};
use crate::{
    board::Board,
    defs::{About, FEN_START_POSITION},
    engine::defs::{EngineOption, EngineSetOption, ErrFatal, Information, UiElement},
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

const PROTOCOL_NAME: &str = "UCI";

#[derive(PartialEq, Clone)]
pub enum UciInput {
    Uci,
    UciNewGame,
    IsReady,
    SetOption(EngineSetOption),
    Position(String, Vec<String>),
    GoInfinite,
    GoDepth(i8),
    GoMoveTime(u128),
    GoNodes(usize),
    GoGameTime(GameTime),
    Stop,
}

#[derive(PartialEq)]
pub enum UciOutput {
    Identify,           // Transmit Uci of the engine.
    Ready,              // Transmit that the engine is ready.
    InfoString(String), // Transmit general information.
}

// This struct is used to instantiate the Comm Console module.
pub struct Uci {
    receiving_handle: Option<JoinHandle<()>>, // Thread for receiving input.
    output_handle: Option<JoinHandle<()>>,    // Thread for sending output.
    output_tx: Option<Sender<CommOutput>>,    // Actual output sender object.
}

// Public functions
impl Uci {
    // Create a new console.
    pub fn new() -> Self {
        Self {
            receiving_handle: None,
            output_handle: None,
            output_tx: None,
        }
    }
}

// Any communication module must implement the trait IComm.
impl IComm for Uci {
    fn init(
        &mut self,
        receiving_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<EngineOption>>,
    ) {
        // Start threads
        self.receiving_thread(receiving_tx);
        self.output_thread(board, options);
    }

    // The engine thread (which is the creator of the Comm module) can use
    // this function to send out of the engine onto the console, or towards
    // a user interface.
    fn send(&self, msg: CommOutput) {
        if let Some(tx) = &self.output_tx {
            tx.send(msg).expect(ErrFatal::CHANNEL);
        }
    }

    // After the engine sends 'quit' to the control thread, it will call
    // wait_for_shutdown() and then wait here until shutdown is completed.
    fn wait_for_shutdown(&mut self) {
        if let Some(h) = self.receiving_handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }

        if let Some(h) = self.output_handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }

    fn info(&self) -> CommType {
        CommType {
            protocol: String::from("UCI"),
            stateful: false,
            fancy_about: true,
        }
    }
}

// Implement the receiving thread
impl Uci {
    // The receiving thread receives incoming commands from the console or
    // GUI, which is turns into a "CommInput" object. It sends this
    // object to the engine thread so the engine can decide what to do.
    fn receiving_thread(&mut self, receiving_tx: Sender<Information>) {
        // Create thread-local variables
        let mut t_incoming_data = String::from(""); // Buffer for incoming data.
        let t_receiving_tx = receiving_tx; // Sends incoming data to engine thread.

        // Actual thread creation.
        let receiving_handle = thread::spawn(move || {
            let mut quit = false;

            // Keep running as long as 'quit' is not detected.
            while !quit {
                // Get data from stdin.
                io::stdin()
                    .read_line(&mut t_incoming_data)
                    .expect(ErrFatal::READ_IO);

                // Create the CommInput object.
                let comm_received = Uci::create_comm_received(&t_incoming_data);

                // Send it to the engine thread.
                t_receiving_tx
                    .send(Information::Comm(comm_received.clone()))
                    .expect(ErrFatal::HANDLE);

                // Terminate the receiving thread if "Quit" was detected.
                quit = comm_received == CommInput::Quit;

                // Clear for next input
                t_incoming_data = String::from("");
            }
        });

        // Store the handle.
        self.receiving_handle = Some(receiving_handle);
    }
}

// Implement receiving/parsing functions
impl Uci {
    // This function turns the incoming data into CommInputs which the
    // engine is able to understand and react to.
    fn create_comm_received(input: &str) -> CommInput {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match i {
            // UCI commands
            cmd if cmd == "uci" => CommInput::Uci(UciInput::Uci),
            cmd if cmd == "ucinewgame" => CommInput::Uci(UciInput::UciNewGame),
            cmd if cmd == "isready" => CommInput::Uci(UciInput::IsReady),
            cmd if cmd == "stop" => CommInput::Uci(UciInput::Stop),
            cmd if cmd.starts_with("setoption") => Uci::parse_setoption(&cmd),
            cmd if cmd.starts_with("position") => Uci::parse_position(&cmd),
            cmd if cmd.starts_with("go") => Uci::parse_go(&cmd),
            cmd if cmd == "quit" || cmd.is_empty() => CommInput::Quit,

            // Custom commands
            cmd if cmd == "board" => CommInput::Board,
            cmd if cmd == "history" => CommInput::History,
            cmd if cmd == "eval" => CommInput::Eval,
            cmd if cmd == "help" => CommInput::Help,

            // Everything else is ignored.
            _ => CommInput::Unknown(i),
        }
    }

    fn parse_position(cmd: &str) -> CommInput {
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

        CommInput::Uci(UciInput::Position(fen.trim().to_string(), moves))
    }

    fn parse_go(cmd: &str) -> CommInput {
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
        let mut comm_received = CommInput::Unknown(String::from(cmd));
        let mut token = Tokens::Nothing;
        let mut game_time = GameTime::new(0, 0, 0, 0, None);

        for p in parts {
            match p {
                t if t == "go" => comm_received = CommInput::Uci(UciInput::GoInfinite),
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
                        comm_received = CommInput::Uci(UciInput::GoDepth(depth));
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::MoveTime => {
                        let milliseconds = p.parse::<u128>().unwrap_or(1000);
                        comm_received = CommInput::Uci(UciInput::GoMoveTime(milliseconds));
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::Nodes => {
                        let nodes = p.parse::<usize>().unwrap_or(1);
                        comm_received = CommInput::Uci(UciInput::GoNodes(nodes));
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
        let is_default_mode = comm_received == CommInput::Uci(UciInput::GoInfinite);
        let has_time = game_time.wtime > 0 || game_time.btime > 0;
        let has_inc = game_time.winc > 0 || game_time.binc > 0;
        let is_game_time = has_time || has_inc;
        if is_default_mode && is_game_time {
            comm_received = CommInput::Uci(UciInput::GoGameTime(game_time));
        }

        comm_received
    } // end parse_go()

    fn parse_setoption(cmd: &str) -> CommInput {
        enum Tokens {
            Nothing,
            Name,
            Value,
        }

        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
        let mut token = Tokens::Nothing;
        let mut name = String::from(""); // Option name provided by the UCI command.
        let mut value = String::from(""); // Option value provided by the UCI command.
        let mut eon = EngineSetOption::Nothing; // Engine Option Name to send to the engine.

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
                "hash" => eon = EngineSetOption::Hash(value),
                "clear hash" => eon = EngineSetOption::ClearHash,
                _ => (),
            }
        }

        // Send the engine option name with value to the engine thread.
        CommInput::Uci(UciInput::SetOption(eon))
    }
}

// Implement the output thread
impl Uci {
    // The control thread receives commands from the engine thread.
    fn output_thread(&mut self, board: Arc<Mutex<Board>>, options: Arc<Vec<EngineOption>>) {
        // Create an incoming channel for the control thread.
        let (output_tx, output_rx) = crossbeam_channel::unbounded::<CommOutput>();

        // Create the output thread.
        let output_handle = thread::spawn(move || {
            let mut quit = false;
            let t_board = Arc::clone(&board);
            let t_options = Arc::clone(&options);

            // Keep running as long as Quit is not received.
            while !quit {
                let output = output_rx.recv().expect(ErrFatal::CHANNEL);

                // Perform command as sent by the engine thread.
                match output {
                    CommOutput::Uci(UciOutput::Identify) => {
                        Uci::id();
                        Uci::options(&t_options);
                        Uci::uciok();
                    }
                    CommOutput::Uci(UciOutput::Ready) => Uci::readyok(),
                    CommOutput::Quit => quit = true, // terminates the output thread.
                    CommOutput::SearchSummary(summary) => Uci::search_summary(&summary),
                    CommOutput::SearchCurrMove(current) => Uci::search_currmove(&current),
                    CommOutput::SearchStats(stats) => Uci::search_stats(&stats),
                    CommOutput::BestMove(bm) => Uci::best_move(bm),
                    CommOutput::Message(msg) => Uci::message(msg),
                    CommOutput::Error(cmd, err_type) => Uci::error(cmd, err_type),

                    // Custom prints for use in the console.
                    CommOutput::PrintBoard => Shared::print_board(&t_board),
                    CommOutput::PrintHistory => Shared::print_history(&t_board),
                    CommOutput::PrintEval(eval, phase) => Shared::print_eval(eval, phase),
                    CommOutput::PrintHelp => Shared::print_help(PROTOCOL_NAME),

                    // Ignore everything else
                    _ => (),
                }
            }
        });

        // Store handle and control sender.
        self.output_handle = Some(output_handle);
        self.output_tx = Some(output_tx);
    }
}

// Implement output functions
impl Uci {
    fn id() {
        println!("id name {} {}", About::ENGINE, About::VERSION);
        println!("id author {}", About::AUTHOR);
    }

    fn options(options: &Arc<Vec<EngineOption>>) {
        for option in options.iter() {
            let name = format!("option name {}", option.name);

            let ui_element = match option.ui_element {
                UiElement::Spin => String::from("type spin"),
                UiElement::Button => String::from("type button"),
            };

            let value_default = if let Some(v) = &option.default {
                format!("default {}", (*v).clone())
            } else {
                String::from("")
            };

            let value_min = if let Some(v) = &option.min {
                format!("min {}", (*v).clone())
            } else {
                String::from("")
            };

            let value_max = if let Some(v) = &option.max {
                format!("max {}", (*v).clone())
            } else {
                String::from("")
            };

            let uci_option = format!(
                "{} {} {} {} {}",
                name, ui_element, value_default, value_min, value_max
            )
            .trim()
            .to_string();

            println!("{}", uci_option);
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

    fn best_move(m: Move) {
        println!("bestmove {}", m.as_string());
    }

    fn message(msg: String) {
        println!("info string {}", msg);
    }

    fn error(cmd: String, err_type: String) {
        println!("info string {}: {}", err_type, cmd);
    }
}
