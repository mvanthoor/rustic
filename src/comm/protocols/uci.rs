// This file implements the UCI communication module.

use crate::{
    board::Board,
    comm::{
        defs::{CommIn, CommInfo, CommOut, CommType, IComm},
        shared::Shared,
    },
    defs::{About, FEN_START_POSITION},
    engine::defs::{
        EngineOption, EngineSetOption, EngineState, ErrFatal, GameResult, Information, UiElement,
    },
    movegen::defs::Move,
    search::defs::{GameTime, SearchCurrentMove, SearchStats, SearchSummary},
};
use crossbeam_channel::{self, Sender};
use std::{
    io,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

// ---------------------------------------------------------------------
// UCI Type definitions
// ---------------------------------------------------------------------

#[derive(PartialEq, Eq, Clone)]
pub enum UciIn {
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

#[derive(PartialEq, Eq)]
pub enum UciOut {
    Identify, // Transmit Uci of the engine.
    Ready,    // Transmit that the engine is ready.
}

// ---------------------------------------------------------------------
// UCI Communication module to be instantiated by the engine
// ---------------------------------------------------------------------

// This struct is used to instantiate the Comm UCI module.
pub struct Uci {
    input_handle: Option<JoinHandle<()>>, // Thread for receiving input.
    output_handle: Option<JoinHandle<()>>, // Thread for sending output.
    output_tx: Option<Sender<CommOut>>,   // Actual output sender object.
    info: CommInfo,
}

// Public functions
impl Uci {
    // Create a new console.
    pub fn new() -> Self {
        Self {
            input_handle: None,
            output_handle: None,
            output_tx: None,
            info: CommInfo::new(CommType::UCI, true, false, false, EngineState::Waiting),
        }
    }
}

// ---------------------------------------------------------------------
// Communication interface: must be implemented by all comm modules.
// ---------------------------------------------------------------------

// Any communication module must implement the trait IComm.
impl IComm for Uci {
    fn init(
        &mut self,
        input_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<EngineOption>>,
    ) {
        // Start threads
        self.input_thread(input_tx);
        self.output_thread(board, options);
    }

    // The engine thread (which is the creator of the Comm module) can use
    // this function to send out of the engine onto the console, or towards
    // a user interface.
    fn send(&self, msg: CommOut) {
        if let Some(tx) = &self.output_tx {
            tx.send(msg).expect(ErrFatal::CHANNEL);
        }
    }

    // After the engine sends 'quit' to the control thread, it will call
    // shutdown() and then wait here until shutdown is completed.
    fn shutdown(&mut self) {
        if let Some(h) = self.input_handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }

        if let Some(h) = self.output_handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }

    fn info(&self) -> &CommInfo {
        &self.info
    }
}

// ---------------------------------------------------------------------
// UCI input thread
// ---------------------------------------------------------------------

// Implement the input thread
impl Uci {
    // The input thread receives incoming commands from the console or
    // GUI, which is turns into a "CommIn" object. It sends this object to
    // the engine thread so the engine can decide what to do.
    fn input_thread(&mut self, transmitter: Sender<Information>) {
        // Create thread-local variables
        let mut incoming_data = String::from(""); // Buffer for incoming data.

        // Actual thread creation.
        let input_handle = thread::spawn(move || {
            let mut quit = false;

            // Keep running as long as 'quit' is not detected.
            while !quit {
                // Get data from stdin.
                io::stdin()
                    .read_line(&mut incoming_data)
                    .expect(ErrFatal::READ_IO);

                // Create the CommIn object.
                let comm_received = Uci::create_comm_input(&incoming_data);

                // Send it to the engine thread.
                transmitter
                    .send(Information::Comm(comm_received.clone()))
                    .expect(ErrFatal::HANDLE);

                // Terminate the receiving thread if "Quit" was detected.
                quit = comm_received == CommIn::Quit;

                // Clear for next input
                incoming_data = String::from("");
            }
        });

        // Store the handle.
        self.input_handle = Some(input_handle);
    }
}

// ---------------------------------------------------------------------
// Parsing functions for the input thread
// ---------------------------------------------------------------------

// Implement receiving/parsing functions
impl Uci {
    // This function turns the incoming data into CommIns which the
    // engine is able to understand and react to.
    fn create_comm_input(input: &str) -> CommIn {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match i {
            // UCI commands
            cmd if cmd == "uci" => CommIn::Uci(UciIn::Uci),
            cmd if cmd == "ucinewgame" => CommIn::Uci(UciIn::UciNewGame),
            cmd if cmd == "isready" => CommIn::Uci(UciIn::IsReady),
            cmd if cmd == "stop" => CommIn::Uci(UciIn::Stop),
            cmd if cmd == "quit" || cmd.is_empty() => CommIn::Quit,
            cmd if cmd.starts_with("setoption") => Uci::parse_setoption(&cmd),
            cmd if cmd.starts_with("position") => Uci::parse_position(&cmd),
            cmd if cmd.starts_with("go") => Uci::parse_go(&cmd),

            // Custom commands
            cmd if cmd == "board" => CommIn::Board,
            cmd if cmd == "history" => CommIn::History,
            cmd if cmd == "eval" => CommIn::Eval,
            cmd if cmd == "state" => CommIn::State,
            cmd if cmd == "cleartt" => CommIn::ClearTt,
            cmd if cmd == "help" => CommIn::Help,

            // Everything else is ignored.
            _ => CommIn::Unknown(i),
        }
    }

    fn parse_position(cmd: &str) -> CommIn {
        enum Tokens {
            Nothing,
            Fen,
            Moves,
        }

        let parts: Vec<&str> = cmd.split_whitespace().collect();
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
                        fen.push_str(p);
                        fen.push(' ');
                    }
                    Tokens::Moves => moves.push(String::from(p)),
                },
            }
        }
        // No FEN part in the command. Use the start position.
        if fen.is_empty() {
            fen = String::from(FEN_START_POSITION)
        }

        CommIn::Uci(UciIn::Position(fen.trim().to_string(), moves))
    }

    fn parse_go(cmd: &str) -> CommIn {
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

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let mut comm_received = CommIn::Unknown(cmd.to_string());
        let mut token = Tokens::Nothing;
        let mut game_time = GameTime::new(0, 0, 0, 0, None);

        for p in parts {
            match p {
                t if t == "go" => comm_received = CommIn::Uci(UciIn::GoInfinite),
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
                        comm_received = CommIn::Uci(UciIn::GoDepth(depth));
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::MoveTime => {
                        let milliseconds = p.parse::<u128>().unwrap_or(1000);
                        comm_received = CommIn::Uci(UciIn::GoMoveTime(milliseconds));
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::Nodes => {
                        let nodes = p.parse::<usize>().unwrap_or(1);
                        comm_received = CommIn::Uci(UciIn::GoNodes(nodes));
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
        let is_default_mode = comm_received == CommIn::Uci(UciIn::GoInfinite);
        let has_time = game_time.wtime > 0 || game_time.btime > 0;
        let has_inc = game_time.winc > 0 || game_time.binc > 0;
        let is_game_time = has_time || has_inc;
        if is_default_mode && is_game_time {
            comm_received = CommIn::Uci(UciIn::GoGameTime(game_time));
        }

        comm_received
    } // end parse_go()

    fn parse_setoption(cmd: &str) -> CommIn {
        enum Tokens {
            Nothing,
            Name,
            Value,
        }

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let mut token = Tokens::Nothing;
        let mut name = String::from(""); // Option name provided by the UCI command.
        let mut value = String::from(""); // Option value provided by the UCI command.
        let mut engine_option_name = EngineSetOption::Nothing; // Name for Engine thread.

        for p in parts {
            match p {
                t if t == "setoption" => (),
                t if t == "name" => token = Tokens::Name,
                t if t == "value" => token = Tokens::Value,
                _ => match token {
                    Tokens::Name => name = format!("{name} {p}"),
                    Tokens::Value => value = p.to_lowercase(),
                    Tokens::Nothing => (),
                },
            }
        }

        // Determine which engine option name to send.
        if !name.is_empty() {
            match name.to_lowercase().trim() {
                "hash" => engine_option_name = EngineSetOption::Hash(value),
                "clear hash" => engine_option_name = EngineSetOption::ClearHash,
                _ => (),
            }
        }

        // Send the engine option name with value to the engine thread.
        CommIn::Uci(UciIn::SetOption(engine_option_name))
    }
}

// ---------------------------------------------------------------------
// UCI output thread
// ---------------------------------------------------------------------

// Implement the output thread
impl Uci {
    // The control thread receives commands from the engine thread.
    fn output_thread(&mut self, board: Arc<Mutex<Board>>, options: Arc<Vec<EngineOption>>) {
        // Create an incoming channel for the output thread.
        let (output_tx, output_rx) = crossbeam_channel::unbounded::<CommOut>();

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
                    CommOut::Uci(UciOut::Identify) => {
                        Uci::id();
                        Uci::options(&t_options);
                        Uci::uciok();
                    }
                    CommOut::Uci(UciOut::Ready) => Uci::readyok(),

                    CommOut::Quit => quit = true, // terminates the output thread.
                    CommOut::Message(msg) => Uci::message(&msg),
                    CommOut::SearchSummary(summary) => Uci::search_summary(&summary),
                    CommOut::SearchCurrMove(current) => Uci::search_currmove(&current),
                    CommOut::SearchStats(stats) => Uci::search_stats(&stats),
                    CommOut::BestMove(bm, result) => Uci::best_move(&bm, &result),
                    CommOut::Error(err_type, cmd) => Uci::error(err_type, &cmd),

                    // Custom commands
                    CommOut::PrintBoard => Shared::print_board(&t_board),
                    CommOut::PrintHistory => Shared::print_history(&t_board),
                    CommOut::PrintEval(eval, phase) => Shared::print_eval(eval, phase),
                    CommOut::PrintState(state) => Shared::print_state(&state),
                    CommOut::PrintHelp => Shared::print_help(CommType::UCI),

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

// ---------------------------------------------------------------------
// Print functions for the output thread
// ---------------------------------------------------------------------

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

            let uci_option = format!("{name} {ui_element} {value_default} {value_min} {value_max}")
                .trim()
                .to_string();

            println!("{uci_option}");
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
        let score = if let Some(moves) = Shared::moves_to_checkmate(s.cp) {
            // If the engine is being mated itself, flip the score.
            let flip = if s.cp < 0 { -1 } else { 1 };
            format!("mate {}", moves * flip)
        } else {
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

        let info = format!(
            "info score {} {} time {} nodes {} nps {}{}pv {}",
            score,
            depth,
            s.time,
            s.nodes,
            s.nps,
            hash_full,
            s.pv_to_string(),
        );

        println!("{info}");
    }

    fn search_currmove(c: &SearchCurrentMove) {
        println!(
            "info currmove {} currmovenumber {}",
            c.curr_move, c.curr_move_number
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

    // Sends the best move to the GUI. GameResult is not used by the
    // UCI-protocol because it depends on the GUI to determine this.
    // Therefore this parameter is ignored.
    fn best_move(m: &Move, _: &Option<GameResult>) {
        println!("bestmove {m}");
    }

    fn message(msg: &str) {
        println!("info string {msg}");
    }

    fn error(err_type: &str, cmd: &str) {
        println!("info string ERROR {err_type}: {cmd}");
    }
}
