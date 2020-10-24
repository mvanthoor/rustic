// This file implements the UCI communication module.

use super::{CommControl, CommReport, CommType, IComm};
use crate::{
    board::{defs::SQUARE_NAME, Board},
    defs::{About, FEN_START_POSITION},
    engine::defs::{ErrFatal, Information},
    misc::print,
    movegen::defs::Move,
    search::defs::{SearchCurrentMove, SearchStats, SearchSummary},
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
    // Uci commands
    Uci,
    IsReady,
    Position(String, Vec<String>),
    GoInfinite,
    Stop,
    Quit,

    // Custom commands
    Board,
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
    fn init(&mut self, report_tx: Sender<Information>, board: Arc<Mutex<Board>>) {
        // Start threads
        self.report_thread(report_tx.clone());
        self.control_thread(board);
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

// This block implements the Report and Control threads.
impl Uci {
    // The Report thread sends incoming data to the engine thread.
    fn report_thread(&mut self, report_tx: Sender<Information>) {
        // Create thread-local variables
        let mut t_incoming_data = String::from("");
        let t_report_tx = report_tx.clone(); // Report sender

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

    // The control thread receives commands from the engine thread.
    fn control_thread(&mut self, board: Arc<Mutex<Board>>) {
        // Create an incoming channel for the control thread.
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<CommControl>();

        // Create the control thread.
        let control_handle = thread::spawn(move || {
            let mut quit = false;
            let t_board = Arc::clone(&board);

            // Keep running as long as Quit is not received.
            while !quit {
                let control = control_rx.recv().expect(ErrFatal::CHANNEL);

                // Perform command as sent by the engine thread.
                match control {
                    CommControl::Identify => {
                        Uci::id();
                        Uci::uciok();
                    }
                    CommControl::Ready => Uci::readyok(),
                    CommControl::Quit => quit = true,
                    CommControl::SearchSummary(summary) => Uci::search_summary(&summary),
                    CommControl::SearchCurrent(current) => Uci::search_current(&current),
                    CommControl::SearchStats(stats) => Uci::search_stats(&stats),
                    CommControl::InfoString(msg) => Uci::info_string(&msg),
                    CommControl::BestMove(bm) => Uci::best_move(&bm),

                    // Custom prints for use in the console.
                    CommControl::PrintBoard => Uci::print_board(&t_board),
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
            cmd if cmd == "isready" => CommReport::Uci(UciReport::IsReady),
            cmd if cmd == "stop" => CommReport::Uci(UciReport::Stop),
            cmd if cmd == "quit" || cmd == "exit" => CommReport::Uci(UciReport::Quit),
            cmd if cmd.starts_with("position startpos") => Uci::parse_position_startpos(&cmd),
            cmd if cmd.starts_with("position fen") => Uci::parse_position_fen(&cmd),
            cmd if cmd.starts_with("go") => Uci::parse_go(&cmd),

            // Custom commands
            cmd if cmd == "board" => CommReport::Uci(UciReport::Board),
            cmd if cmd == "eval" => CommReport::Uci(UciReport::Eval),
            cmd if cmd == "help" => CommReport::Uci(UciReport::Help),

            // Everything else is ignored.
            _ => CommReport::Uci(UciReport::Unknown),
        }
    }

    // Parses "position startpos [moves ...]"
    fn parse_position_startpos(cmd: &String) -> CommReport {
        // Cut off "position startpos", so ony "moves ..." remains.
        let cmd = cmd.replace("position startpos", "").trim().to_string();
        let fen = FEN_START_POSITION.to_string();
        let moves: Vec<String> = Uci::parse_position_moves(&cmd);

        CommReport::Uci(UciReport::Position(fen, moves))
    }

    // Parses "position fen <fen_string> [moves ...]"
    fn parse_position_fen(cmd: &String) -> CommReport {
        // Cut "position fen" from the beginning of the command.
        let cmd = cmd.replace("position fen", "").trim().to_string();

        // Split into "fen" and "moves" part.
        let parts: Vec<String> = cmd.split("moves").map(|s| s.trim().to_string()).collect();

        let (fen, moves): (String, Vec<String>) = match parts.len() {
            0 => (String::from(""), Vec::<String>::new()),
            1 => (parts[0].clone(), Vec::<String>::new()),
            _ => {
                let m = format!("moves {}", &parts[1]);
                (parts[0].clone(), Uci::parse_position_moves(&m))
            }
        };

        CommReport::Uci(UciReport::Position(fen, moves))
    }

    fn parse_position_moves(part: &String) -> Vec<String> {
        const SPACE: &str = " ";

        // If the word "moves" is in this string, then remove it and
        // transform the moves after it into a list.
        if part.starts_with("moves") {
            let part = part.replace("moves", "").trim().to_string();
            let list: Vec<String> = part.split(SPACE).map(|m| m.to_string()).collect();
            list
        } else {
            Vec::<String>::new()
        }
    }

    fn parse_go(cmd: &String) -> CommReport {
        let cmd = cmd.replace("go", "").trim().to_string();
        let mut report = CommReport::Uci(UciReport::Unknown);
        let go_infinite = cmd == "infinite" || cmd.is_empty();

        if go_infinite {
            report = CommReport::Uci(UciReport::GoInfinite);
        }
        report
    }
}

// Implements UCI responses to send to the G(UI).
impl Uci {
    fn id() {
        println!("id name {} {}", About::ENGINE, About::VERSION);
        println!("id author {}", About::AUTHOR);
    }

    fn uciok() {
        println!("uciok");
    }

    fn readyok() {
        println!("readyok");
    }

    fn search_summary(s: &SearchSummary) {
        let pv_move = format!(
            "{}{}",
            SQUARE_NAME[s.bm_at_depth.from()],
            SQUARE_NAME[s.bm_at_depth.to()]
        );

        let info = format!(
            "info score cp {} depth {} time {} nodes {} nps {} pv {}",
            s.cp, s.depth, s.time, s.nodes, s.nps, pv_move,
        );

        println!("{}", info);
    }

    fn search_current(c: &SearchCurrentMove) {
        println!(
            "info currmove {}{} currmovenumber {}",
            SQUARE_NAME[c.curr_move.from()],
            SQUARE_NAME[c.curr_move.to()],
            c.curr_move_number
        );
    }

    fn search_stats(s: &SearchStats) {
        println!("info nodes {} nps {}", s.nodes, s.nps);
    }

    fn info_string(msg: &String) {
        println!("info string {}", msg);
    }

    fn best_move(bm: &Move) {
        println!(
            "bestmove {}{}",
            SQUARE_NAME[bm.from()],
            SQUARE_NAME[bm.to()]
        );
    }
}

// implements handling of custom commands. These are mostly used when using
// the UCI protocol directly in a terminal window.
impl Uci {
    fn print_board(board: &Arc<Mutex<Board>>) {
        print::position(&board.lock().expect(ErrFatal::LOCK), None);
    }

    fn print_help() {
        println!("The engine is in UCI communication mode. It supports some custom");
        println!("non-UCI commands to make use through a terminal window easier.");
        println!();
        println!("Custom commands");
        println!("================================================================");
        println!("help      :   This help information.");
        println!("board     :   Print the current board state.");
        println!("eval      :   Print evaluation for side to move.");
        println!("exit      :   Quit/Exit the engine.");
        println!();
    }
}
