// This file implements the UCI communication module.

use super::{CommControl, CommReport, CommType, IComm};
use crate::{
    board::Board,
    defs::{About, FEN_START_POSITION},
    engine::defs::{ErrFatal, Information},
    misc::print,
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
    Quit,

    // Custom commands
    Board,
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
                    CommControl::PrintBoard => Uci::print_board(&t_board),
                    CommControl::PrintHelp => Uci::print_help(),
                    CommControl::PrintMessage(msg) => println!("{}", msg),

                    // Comm Control commands that are not (yet) used.
                    CommControl::PrintBestMove(_)
                    | CommControl::PrintSearchSummary(_)
                    | CommControl::Update => (),
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
            cmd if cmd == "quit" || cmd == "exit" => CommReport::Uci(UciReport::Quit),
            cmd if cmd.starts_with("position startpos") => Uci::parse_position_startpos(&cmd),
            cmd if cmd.starts_with("position fen") => Uci::parse_position_fen(&cmd),

            // Custom commands
            cmd if cmd == "board" => CommReport::Uci(UciReport::Board),
            cmd if cmd == "help" => CommReport::Uci(UciReport::Help),

            // Everything else is ignored.
            _ => CommReport::Uci(UciReport::Unknown),
        }
    }

    fn parse_position_startpos(cmd: &String) -> CommReport {
        let cmd = cmd.replace("position startpos", "").trim().to_string();
        let fen = FEN_START_POSITION.to_string();
        let moves: Vec<String> = Uci::parse_position_moves(&cmd);

        CommReport::Uci(UciReport::Position(fen, moves))
    }

    fn parse_position_fen(cmd: &String) -> CommReport {
        let cmd = cmd.replace("position fen", "").trim().to_string();
        let mut fen = String::from("");
        for c in cmd.chars() {
            fen.push(c);
        }
        fen.trim().to_string();
        CommReport::Uci(UciReport::Position(fen, Vec::<String>::new()))
    }

    fn parse_position_moves(cmd: &String) -> Vec<String> {
        const SPACE: &str = " ";

        // Return the moves or an empty vector. The engine will determine
        // by itself if the moves are actually legal.
        if cmd.starts_with("moves") {
            let cmd = cmd.replace("moves", "").trim().to_string();
            let algebraic_moves: Vec<String> = cmd.split(SPACE).map(|m| m.to_string()).collect();
            algebraic_moves
        } else {
            Vec::new()
        }
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
        println!("exit      :   Quit/Exit the engine.");
        println!();
    }
}
