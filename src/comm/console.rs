// This file implements the Console interface. In this communication mode,
// the engine shows the current board position in the terminal, and it will
// accept commands typed by the user. This interface is mainly used for
// engine development, but it can also be used to (laboriously) play a
// complete game.

use super::{CommControl, CommReport, CommType, GeneralReport, IComm};
use crate::{
    board::{defs::SQUARE_NAME, Board},
    defs::About,
    engine::defs::{ErrFatal, Information},
    misc::print,
    search::defs::SearchSummary,
};
use crossbeam_channel::{self, Sender};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

type HelpLine = (&'static str, &'static str, &'static str);
const UNKNOWN_INPUT: &'static str = "Unknown input:";

// Input will be turned into a report, which wil be sent to the engine. The
// main engine thread will react accordingly.
#[derive(PartialEq, Clone)]
pub enum ConsoleReport {
    Search,
    Cancel,
    Move(String),
    Evaluate,
    Takeback,
}

// This struct is used to instantiate the Comm Console module.
pub struct Console {
    control_handle: Option<JoinHandle<()>>,
    report_handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<CommControl>>,
    last_report: Arc<Mutex<CommReport>>,
}

// Public functions
impl Console {
    // Create a new console.
    pub fn new() -> Self {
        let nothing = CommReport::General(GeneralReport::Nothing);
        Self {
            control_handle: None,
            report_handle: None,
            control_tx: None,
            last_report: Arc::new(Mutex::new(nothing)),
        }
    }
}

// Any communication module must implement the trait IComm.
impl IComm for Console {
    fn init(&mut self, report_tx: Sender<Information>, board: Arc<Mutex<Board>>) {
        // Start threads
        self.report_thread(report_tx.clone(), Arc::clone(&board));
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
        CommType::CONSOLE
    }
}

// This block implements the Report and Control threads.
impl Console {
    // The Report thread sends incoming data to the engine thread.
    fn report_thread(&mut self, report_tx: Sender<Information>, board: Arc<Mutex<Board>>) {
        // Create thread-local variables
        let mut t_incoming_data = String::from("");
        let t_report_tx = report_tx.clone(); // Report sender
        let t_board = Arc::clone(&board);
        let t_last_report = Arc::clone(&self.last_report);

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
                let new_report = Console::create_report(&t_incoming_data);

                // Check if the created report is valid, so it is something
                // the engine will understand.
                if new_report.is_valid() {
                    // Valid. Save as last report.
                    *t_last_report.lock().expect(ErrFatal::LOCK) = new_report.clone();

                    // Send it to the engine thread.
                    t_report_tx
                        .send(Information::Comm(new_report.clone()))
                        .expect(ErrFatal::HANDLE);

                    // Terminate the reporting thread if "Quit" was detected.
                    quit = new_report == CommReport::General(GeneralReport::Quit);
                } else {
                    // Not a valid report. Save "Unkown" as last report.
                    let unknown = CommReport::General(GeneralReport::Unknown);
                    *t_last_report.lock().expect(ErrFatal::LOCK) = unknown;

                    // Print an error.
                    print!("{} {}", UNKNOWN_INPUT, t_incoming_data);

                    // Update the screen.
                    Console::update(&t_last_report, &t_board);
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

        // Create thread-local variables.
        let t_last_report = Arc::clone(&self.last_report);

        // Create the control thread.
        let control_handle = thread::spawn(move || {
            let mut quit = false;

            // Keep running as long as Quit is not received.
            while !quit {
                let control = control_rx.recv().expect(ErrFatal::CHANNEL);

                match control {
                    CommControl::Quit => quit = true,
                    CommControl::Update => Console::update(&t_last_report, &board),
                    CommControl::PrintHelp => Console::print_help(),
                    CommControl::PrintBestMove(m) => {
                        println!("bestmove: {}{}", SQUARE_NAME[m.from()], SQUARE_NAME[m.to()])
                    }
                    CommControl::Print(msg) => println!("{}", msg),
                    CommControl::PrintEvaluation(eval) => Console::print_evaluation(eval),
                    CommControl::PrintSearchSummary(summary) => {
                        Console::print_search_summary(summary)
                    }
                }
            }
        });

        // Store handle and control sender.
        self.control_handle = Some(control_handle);
        self.control_tx = Some(control_tx);
    }
}

// Private functions for this module.
impl Console {
    fn update(last_report: &Arc<Mutex<CommReport>>, board: &Arc<Mutex<Board>>) {
        match *last_report.lock().expect(ErrFatal::LOCK) {
            CommReport::General(GeneralReport::Nothing)
            | CommReport::Console(ConsoleReport::Move(_))
            | CommReport::Console(ConsoleReport::Takeback) => {
                Console::print_position(board);
                Console::print_prompt();
            }
            _ => Console::print_prompt(),
        }
    }

    // This function transforms the typed characters into a command tht the
    // engine which is running in the main thread can understand.
    fn create_report(input: &str) -> CommReport {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match &i[..] {
            "help" | "h" => CommReport::General(GeneralReport::Help),
            "search" | "s" => CommReport::Console(ConsoleReport::Search),
            "cancel" | "c" => CommReport::Console(ConsoleReport::Cancel),
            "evaluate" | "e" => CommReport::Console(ConsoleReport::Evaluate),
            "takeback" | "t" => CommReport::Console(ConsoleReport::Takeback),
            "quit" | "q" => CommReport::General(GeneralReport::Quit),
            "exit" | "x" => CommReport::General(GeneralReport::Quit),
            _ => CommReport::Console(ConsoleReport::Move(i)),
        }
    }
}

// Printing functions.
impl Console {
    const DIVIDER_LENGTH: usize = 48;
    const HELP_UNDERLINE: usize = 65;
    const PROMPT: &'static str = ">";
    const HELP: [HelpLine; 8] = [
        ("a6a7q", " ", "Moves in long algebraic notation."),
        ("help", "h", "This help information."),
        ("search", "s", "Start searching for the best move."),
        ("takeback", "t", "Take back the last move."),
        ("cancel", "c", "Cancel search and report the best move."),
        ("evaluate", "e", "Evaluate the position."),
        ("quit", "q", "Quit the console."),
        ("exit", "x", "Quit the console."),
    ];

    fn print_help() {
        println!("The console supports both long and short commands:\n");
        println!("{:<12}{:<10}{}", "Long", "Short", "Description");
        println!("{}", "=".repeat(Console::HELP_UNDERLINE));
        for line in Console::HELP.iter() {
            println!("{:<12}{:<10}{}", line.0, line.1, line.2);
        }
        println!();
    }

    // Some protocols require output before reading; in the case of
    // "console", the board position and prompt must be printed.
    fn print_position(board: &Arc<Mutex<Board>>) {
        println!("{}", "=".repeat(Console::DIVIDER_LENGTH));
        print::position(&board.lock().expect(ErrFatal::LOCK), None);
    }

    // This function creates the engine's command prompt.
    fn print_prompt() {
        print!("{} {} ", About::ENGINE, Console::PROMPT);
        io::stdout().flush().expect(ErrFatal::FLUSH_IO);
    }

    // This function prints the evaluation from White's point of view.
    fn print_evaluation(eval: i16) {
        println!("Evaluation: {}", eval);
    }

    fn print_search_summary(s: SearchSummary) {
        let seconds = s.time as f64 / 1_000f64;
        let knps = (s.nps as f64 / 1_000f64).round() as usize;
        println!(
            "depth: {}, bestmove: {}{}, eval: {}, time: {}s, nodes: {}, knps: {}",
            s.depth,
            SQUARE_NAME[s.curr_move.from()],
            SQUARE_NAME[s.curr_move.to()],
            s.cp, // centipawns
            seconds,
            s.nodes,
            knps
        );
    }
}
