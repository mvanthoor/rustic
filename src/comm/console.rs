// This file implements the Console interface. In this communication mode,
// the engine shows the current board position in the terminal, and it will
// accept commands typed by the user. This interface is mainly used for
// engine development.

use super::{CommControl, CommReport, CommType, IComm};
use crate::{
    board::Board,
    defs::About,
    engine::defs::{ErrFatal, Information},
    misc::print,
};
use crossbeam_channel::{self, Sender};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

const UNKNOWN_INPUT: &'static str = "Unknown input:";

pub struct Console {
    control_handle: Option<JoinHandle<()>>,
    report_handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<CommControl>>,
    last_report: Arc<Mutex<CommReport>>,
}

// Public functions
impl Console {
    pub fn new() -> Self {
        Self {
            control_handle: None,
            report_handle: None,
            control_tx: None,
            last_report: Arc::new(Mutex::new(CommReport::Nothing)),
        }
    }
}

// Any communication module must implement the trait IComm.
impl IComm for Console {
    fn init(&mut self, report_tx: Sender<Information>, board: Arc<Mutex<Board>>) {
        // Start threads
        self.report_thread(report_tx.clone(), Arc::clone(&board));
        self.control_thread(board);

        // Report to engine that init is finished.
        report_tx
            .send(Information::Comm(CommReport::InitCompleted))
            .expect(ErrFatal::CHANNEL);
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
                // Get data from stdin and create a report from it.
                io::stdin()
                    .read_line(&mut t_incoming_data)
                    .expect(ErrFatal::READ_IO);
                let new_report = Console::create_report(&t_incoming_data);

                // Check validity of the created report and act accordingly.
                if new_report.is_valid() {
                    // Valid. Save as last report, and send it to the engine.
                    *t_last_report.lock().expect(ErrFatal::LOCK) = new_report.clone();
                    t_report_tx
                        .send(Information::Comm(new_report.clone()))
                        .expect(ErrFatal::HANDLE);
                    // Terminate if 'CommReport::Quit' was sent.
                    quit = new_report == CommReport::Quit;
                } else {
                    // Or give an error message, and update the screen.
                    print!("{} {}", UNKNOWN_INPUT, t_incoming_data);
                    *t_last_report.lock().expect(ErrFatal::LOCK) = CommReport::Unknown;
                    Console::update(&t_last_report, &t_board);
                }
                // Clear for next input
                t_incoming_data = String::from("");
            }
        });
        // Store the handle.
        self.report_handle = Some(report_handle);
    }

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
                    CommControl::Write(msg) => println!("{}", msg),
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
    const DIVIDER_LENGTH: usize = 48;
    const PROMPT: &'static str = ">";

    fn update(last_report: &Arc<Mutex<CommReport>>, board: &Arc<Mutex<Board>>) {
        match *last_report.lock().expect(ErrFatal::LOCK) {
            CommReport::Nothing | CommReport::Move(_) | CommReport::InitCompleted => {
                Console::print_position(board);
                Console::print_prompt();
            }
            _ => Console::print_prompt(),
        }
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

    // This function transforms the typed characters into a command tht the
    // engine which is running in the main thread can understand.
    fn create_report(input: &str) -> CommReport {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match &i[..] {
            "quit" | "exit" => CommReport::Quit,
            "start" => CommReport::Start,
            "stop" => CommReport::Stop,
            "e" => CommReport::Evaluate,
            _ => CommReport::Move(i),
        }
    }
}
