/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2022, Marcel Vanthoor
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

// This file implements the XBoard communication protocol.

use crate::{
    board::Board,
    comm::{
        defs::{CommIn, CommInfo, CommOut, CommType, FancyAbout, IComm, Stateful},
        shared::Shared,
    },
    defs::{About, Sides},
    engine::defs::{EngineOption, EngineState, ErrFatal, GameOverReason, GameResult, Information},
    movegen::defs::Move,
    search::defs::{SearchCurrentMove, SearchStats, SearchSummary},
};
use crossbeam_channel::{self, Sender};
use std::{
    fmt::{self, Display},
    io::{self},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

// ---------------------------------------------------------------------
// XBOARD Type definitions
// ---------------------------------------------------------------------

// We send this list of features to the user interface, so it knows which
// features are supported by the engine. The most important features in
// this list are "memory=1" (hash table), and "draw=0" (we don't accept or
// offer draws and play all games until the end).
const FEATURES: [&str; 12] = [
    "done=0",
    "myname=x",
    "ping=1",
    "memory=1",
    "setboard=1",
    "usermove=1",
    "debug=1",
    "draw=0",
    "ics=0",
    "sigint=0",
    "sigterm=0",
    "done=1",
];

const MPS_PLAYER: usize = 0;
const MPS_ENGINE: usize = 1;
const CHECKMATE_VALUE: i32 = 100_000;

// This struct buffers statistics output. In XBoard, the engine does not
// continuously send intermediate stat updates such as the current move.
// The GUI asks for these stats by sending the Dot-command, and the engine
// has to buffer the stats until the GUI wants them.
struct Stat01 {
    depth: i8,
    time: u128,
    nodes: usize,
    curr_move: Move,
    curr_move_number: u8,
    legal_moves_total: u8,
}
impl Stat01 {
    fn new() -> Self {
        Self {
            depth: 0,
            time: 0,
            nodes: 0,
            curr_move: Move::new(0),
            curr_move_number: 0,
            legal_moves_total: 0,
        }
    }
}

// In XBoard, the GUI does not prompt the engine that it is its turn to
// move by sending a position and time controls. It sends the time controls
// at the beginning of the game and expects the engine to know when it is
// its move, and how much time is left. This struct stores the time control
// informatin as sent by the GUI at the beginning of the game.
#[derive(PartialEq, Clone)]
pub struct TimeControl {
    move_depth: i8,
    move_time: u128,
    moves_per_session: [u8; Sides::BOTH],
    base_time: u128,
    increment: u128,
    is_move_time: bool,
    is_game_time: bool,
}

impl TimeControl {
    fn new() -> Self {
        Self {
            move_depth: 0,
            move_time: 0,
            moves_per_session: [0, 0],
            base_time: 0,
            increment: 0,
            is_move_time: false,
            is_game_time: false,
        }
    }

    pub fn depth(&self) -> i8 {
        self.move_depth
    }

    pub fn move_time(&self) -> u128 {
        self.move_time
    }

    pub fn is_move_time(&self) -> bool {
        self.is_move_time
    }

    pub fn is_game_time(&self) -> bool {
        self.is_game_time
    }

    pub fn is_set(&self) -> bool {
        *self != TimeControl::new()
    }
}

impl Display for TimeControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "sd: {} st: {} mps: {}/{} bt: {} inc: {}",
            self.move_depth,
            self.move_time,
            self.moves_per_session[MPS_PLAYER],
            self.moves_per_session[MPS_ENGINE],
            self.base_time,
            self.increment,
        )
    }
}

// This is a list of supported incoming XBoard commands.
#[derive(PartialEq, Clone)]
pub enum XBoardIn {
    XBoard,
    ProtoVer(u8),
    New,
    Force,
    Go(TimeControl),
    QuestionMark,
    SetBoard(String),
    UserMove(String, TimeControl),
    Undo,
    Remove,
    Result(String, String),
    Ping(i8),
    Post,
    NoPost,
    Memory(usize),
    Analyze,
    Dot,
    Exit,
    Buffered(XBoardInBuffered),
}

// Define how commands are printed in case they need to be, for example
// when an error is detected.
impl Display for XBoardIn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XBoardIn::XBoard => write!(f, "xboard"),
            XBoardIn::ProtoVer(version) => write!(f, "protover {}", version),
            XBoardIn::New => write!(f, "new"),
            XBoardIn::Force => write!(f, "force"),
            XBoardIn::Go(tc) => write!(f, "go {}", tc),
            XBoardIn::QuestionMark => write!(f, "?"),
            XBoardIn::SetBoard(fen) => write!(f, "setboard {}", fen),
            XBoardIn::UserMove(mv, tc) => write!(f, "usermove {} {}", mv, tc),
            XBoardIn::Undo => write!(f, "undo"),
            XBoardIn::Remove => write!(f, "remove"),
            XBoardIn::Result(result, reason) => write!(f, "result {} {{{}}}", result, reason),
            XBoardIn::Ping(count) => write!(f, "ping {}", count),
            XBoardIn::Post => write!(f, "post"),
            XBoardIn::NoPost => write!(f, "nopost"),
            XBoardIn::Memory(mb) => write!(f, "memory {}", mb),
            XBoardIn::Analyze => write!(f, "analyze"),
            XBoardIn::Dot => write!(f, "."),
            XBoardIn::Exit => write!(f, "exit"),
            XBoardIn::Buffered(cmd) => write!(f, "{}", cmd),
        }
    }
}

// Some incoming commands are not sent to the engine thread directly. An
// example are the commands regarding time control. These are buffered
// within the XBoard module, to keep the engine unaware of XBoard
// implementation details. An "XBoardInBuffered::sd" command is sent to the
// engine, so it knows (and prints) that an incoming command was received
// and buffered.
#[derive(PartialEq, Clone)]
pub enum XBoardInBuffered {
    Sd(i8),
    St(u128),
    Level(u8, u128, u128),
}

// Same as normal incoming commands: define how they are displayed.
impl Display for XBoardInBuffered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XBoardInBuffered::Sd(depth) => write!(f, "sd {}", depth),
            XBoardInBuffered::St(time) => write!(f, "st {}", time),
            XBoardInBuffered::Level(moves_per_session, base_time, increment) => {
                write!(f, "level {} {} {}", moves_per_session, base_time, increment)
            }
        }
    }
}

// This is a list of supported XBoard output commands. These commands send
// information to the GUI.
pub enum XBoardOut {
    NewLine,
    Features,
    Stat01,
    IllegalMove(String),
    Pong(i8),
}

// ---------------------------------------------------------------------
// XBOARD Communication module to be instantiated by the engine
// ---------------------------------------------------------------------

// This struct is used to instantiate the Comm XBoard module.
pub struct XBoard {
    input_handle: Option<JoinHandle<()>>,
    output_handle: Option<JoinHandle<()>>,
    output_tx: Option<Sender<CommOut>>,
    info: CommInfo,
}

// Public functions
impl XBoard {
    // Create a new console.
    pub fn new() -> Self {
        Self {
            input_handle: None,
            output_handle: None,
            output_tx: None,
            info: CommInfo::new(
                CommType::XBOARD,
                FancyAbout::No,
                Stateful::Yes,
                EngineState::Observing,
            ),
        }
    }
}

// ---------------------------------------------------------------------
// Communication interface: must be implemented by all comm modules.
// ---------------------------------------------------------------------

impl IComm for XBoard {
    fn init(
        &mut self,
        receiving_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<EngineOption>>,
    ) {
        // Start threads
        self.input_thread(receiving_tx);
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
// XBoard input thread
// ---------------------------------------------------------------------

impl XBoard {
    // The input thread receives incoming commands from the console or
    // GUI, which is turns into a "CommIn" object. It sends this
    // object to the engine thread so the engine can decide what to do.
    fn input_thread(&mut self, transmitter: Sender<Information>) {
        let mut incoming_data = String::from("");

        // Incoming time controls are buffered so they can be sent on each
        // new engine turn, just as in the UCI protocol. This way the
        // engine does not have to keep its own time controls.
        let mut buf_tc = TimeControl::new();

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
                let mut comm_in = XBoard::create_comm_input(&incoming_data);

                match comm_in {
                    // Buffer maximum search depth as time control.
                    CommIn::XBoard(XBoardIn::Buffered(XBoardInBuffered::Sd(depth))) => {
                        buf_tc.move_depth = depth;
                    }

                    // Buffer XBoard version of "movetime".
                    CommIn::XBoard(XBoardIn::Buffered(XBoardInBuffered::St(time))) => {
                        // Set "movetime" time control in milliseconds.
                        buf_tc.move_time = time;

                        // Disable "level" time controls.
                        buf_tc.moves_per_session = [0, 0];
                        buf_tc.base_time = 0;
                        buf_tc.increment = 0;
                        buf_tc.is_game_time = false;

                        // But we're only in MoveTime mode if we got a time
                        // above 0.
                        buf_tc.is_move_time = time > 0;
                    }

                    // Buffer the "level" command.
                    CommIn::XBoard(XBoardIn::Buffered(XBoardInBuffered::Level(mps, bt, inc))) => {
                        // Set "level" time controls.
                        buf_tc.moves_per_session = [mps, mps];
                        buf_tc.base_time = bt;
                        buf_tc.increment = inc;

                        // Disable "movetime" time control.
                        buf_tc.move_time = 0;
                        buf_tc.is_move_time = false;

                        // But we are only in GameTime mode if one of the
                        // parameters is larger than 0.
                        buf_tc.is_game_time = mps > 0 || bt > 0 || inc > 0;
                    }

                    // Add the time control to the usermove. This way the
                    // engine has current TC data when its turn comes up
                    // after executing the usermove.
                    CommIn::XBoard(XBoardIn::UserMove(mv, _)) => {
                        comm_in = CommIn::XBoard(XBoardIn::UserMove(mv, buf_tc.clone()));
                    }

                    // Add the time control to the Go command.
                    CommIn::XBoard(XBoardIn::Go(_)) => {
                        comm_in = CommIn::XBoard(XBoardIn::Go(buf_tc.clone()));
                    }
                    _ => (),
                }

                transmitter
                    .send(Information::Comm(comm_in.clone()))
                    .expect(ErrFatal::HANDLE);

                // Terminate the receiving thread if "Quit" was detected.
                quit = comm_in == CommIn::Quit;

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

impl XBoard {
    // This function turns the incoming data into CommIns which the
    // engine is able to understand and react to.
    fn create_comm_input(input: &str) -> CommIn {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match i {
            cmd if cmd == "xboard" => CommIn::XBoard(XBoardIn::XBoard),
            cmd if cmd == "new" => CommIn::XBoard(XBoardIn::New),
            cmd if cmd == "force" => CommIn::XBoard(XBoardIn::Force),
            cmd if cmd == "go" => CommIn::XBoard(XBoardIn::Go(TimeControl::new())),
            cmd if cmd == "undo" => CommIn::XBoard(XBoardIn::Undo),
            cmd if cmd == "remove" => CommIn::XBoard(XBoardIn::Remove),
            cmd if cmd == "?" => CommIn::XBoard(XBoardIn::QuestionMark),
            cmd if cmd == "post" => CommIn::XBoard(XBoardIn::Post),
            cmd if cmd == "nopost" => CommIn::XBoard(XBoardIn::NoPost),
            cmd if cmd == "analyze" => CommIn::XBoard(XBoardIn::Analyze),
            cmd if cmd == "." => CommIn::XBoard(XBoardIn::Dot),
            cmd if cmd == "exit" => CommIn::XBoard(XBoardIn::Exit),
            cmd if cmd == "quit" || cmd.is_empty() => CommIn::Quit,
            cmd if cmd.starts_with("ping") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("protover") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("setboard") => XBoard::parse_setboard(&cmd),
            cmd if cmd.starts_with("usermove") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("result") => XBoard::parse_result(&cmd),
            cmd if cmd.starts_with("memory") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("sd") => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("st") && cmd != "state" => XBoard::parse_key_value_pair(&cmd),
            cmd if cmd.starts_with("level") => XBoard::parse_level(&cmd),

            // Custom commands
            cmd if cmd == "board" => CommIn::Board,
            cmd if cmd == "history" => CommIn::History,
            cmd if cmd == "eval" => CommIn::Eval,
            cmd if cmd == "state" => CommIn::State,
            cmd if cmd == "cleartt" => CommIn::ClearTt,
            cmd if cmd == "help" => CommIn::Help,

            // Ignore these commands. May GUI's send them, but the  engine
            // doesn't have any use for them (yet).
            cmd if cmd == "easy" || cmd == "hard" => CommIn::Ignore(cmd),
            cmd if cmd == "random" || cmd.starts_with("accepted") => CommIn::Ignore(cmd),

            // Try to parse anything else as a move.
            _ => XBoard::parse_move(&i),
        }
    }

    // Parse a key value pair. This turns commands such as "memory 32" into
    // the enum Memory(32) which can then be sent to the engine.
    fn parse_key_value_pair(cmd: &str) -> CommIn {
        const KEY: usize = 0;
        const VALUE: usize = 1;
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        // Key-value pair has to have two parts. Ignore anything else after
        // the second part.
        if parts.len() >= 2 {
            match parts[KEY] {
                "ping" => {
                    let value = parts[VALUE].parse::<i8>().unwrap_or(0);
                    CommIn::XBoard(XBoardIn::Ping(value))
                }
                "protover" => {
                    let value = parts[VALUE].parse::<u8>().unwrap_or(0);
                    CommIn::XBoard(XBoardIn::ProtoVer(value))
                }
                "memory" => {
                    let value = parts[VALUE].parse::<usize>().unwrap_or(0);
                    CommIn::XBoard(XBoardIn::Memory(value))
                }
                "usermove" => {
                    let value = parts[VALUE].to_lowercase();
                    CommIn::XBoard(XBoardIn::UserMove(value, TimeControl::new()))
                }
                "sd" => {
                    let value = parts[VALUE].parse::<i8>().unwrap_or(0);
                    CommIn::XBoard(XBoardIn::Buffered(XBoardInBuffered::Sd(value)))
                }
                "st" => {
                    let value = parts[VALUE].parse::<u128>().unwrap_or(0);

                    // Convert to milliseconds
                    CommIn::XBoard(XBoardIn::Buffered(XBoardInBuffered::St(value * 1000)))
                }

                _ => CommIn::Unknown(cmd.to_string()),
            }
        } else {
            CommIn::Unknown(cmd.to_string())
        }
    }

    fn parse_result(cmd: &str) -> CommIn {
        const VALID_RESULTS: [&str; 4] = ["1-0", "0-1", "1/2-1/2", "*"];

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let length = parts.len();
        let space_and_brackets: &[_] = &[' ', '{', '}'];
        let mut reason = String::from("");
        let mut result = String::from("");
        let mut comm_in = CommIn::Unknown(cmd.to_string());

        if length >= 2 {
            for (i, p) in parts.iter().enumerate() {
                match i {
                    0 => continue,
                    1 if VALID_RESULTS.contains(p) => result = p.to_string(),
                    _ if length > 2 => reason = format!("{} {}", reason, p),
                    _ => (),
                }
            }

            reason = reason.trim_matches(space_and_brackets).to_string();
            if reason.is_empty() {
                reason = String::from("(empty)");
            };

            if !result.is_empty() {
                comm_in = CommIn::XBoard(XBoardIn::Result(result, reason));
            }
        };

        comm_in
    }

    fn parse_setboard(cmd: &str) -> CommIn {
        let fen = cmd.replace("setboard", "").trim().to_string();
        CommIn::XBoard(XBoardIn::SetBoard(fen))
    }

    fn parse_level(cmd: &str) -> CommIn {
        const MOVES_PER_SESSION: usize = 1;
        const BASE_TIME: usize = 2;
        const INCREMENT: usize = 3;
        const MINUTES: usize = 0;
        const SECONDS: usize = 1;
        const COLON: &str = ":";
        const PERIOD: &str = ".";

        let parts: Vec<&str> = cmd.split_whitespace().collect();

        // The level command must have at least a length of four parts: "level
        // moves_per_session base_time increment". Anything after
        // "increment" is ignored.
        if parts.len() >= 4 {
            // Set moves per session.
            let mps = parts[MOVES_PER_SESSION].parse::<u8>().unwrap_or(0);

            // Parse base time into milliseconds.
            let bt = if parts[BASE_TIME].contains(COLON) {
                // Split into minutes and seconds
                let time: Vec<&str> = parts[BASE_TIME].split(COLON).collect();
                let minutes = time[MINUTES].parse::<u128>().unwrap_or(0);
                let seconds = time[SECONDS].parse::<u128>().unwrap_or(0);
                ((minutes * 60) + seconds) * 1000
            } else {
                let minutes = parts[BASE_TIME].parse::<u128>().unwrap_or(0);
                minutes * 60 * 1000
            };

            // Return increment in milliseconds.
            let inc = if parts[INCREMENT].contains(PERIOD) {
                let fraction_of_second = parts[INCREMENT].parse::<f64>().unwrap_or(0.0);
                (fraction_of_second * 1000_f64).round() as u128
            } else {
                parts[INCREMENT].parse::<u128>().unwrap_or(0) * 1000
            };

            CommIn::XBoard(XBoardIn::Buffered(XBoardInBuffered::Level(mps, bt, inc)))
        } else {
            CommIn::Unknown(cmd.to_string())
        }
    }

    // Try to parse anything that is not a command as a move.
    fn parse_move(cmd: &str) -> CommIn {
        const ALPHA_COORDS: &str = "abcdefgh";
        const DIGIT_COORDS: &str = "12345678";
        const PROMOTIONS: &str = "qrbn";

        // Count the number of correct characters.
        let mut char_ok = 0;

        // A move is always 4 or 5 characters long.
        if cmd.len() == 4 || cmd.len() == 5 {
            // Check correctness of each character.
            for (i, char) in cmd.chars().enumerate() {
                match i {
                    0 | 2 if ALPHA_COORDS.contains(char) => char_ok += 1,
                    1 | 3 if DIGIT_COORDS.contains(char) => char_ok += 1,
                    4 if PROMOTIONS.contains(char) => char_ok += 1,
                    _ => (),
                }
            }
        }

        // If all characters are OK, then this is a plausible move.
        // Otherwise, it is an unknown command.
        if cmd.len() == char_ok {
            CommIn::XBoard(XBoardIn::UserMove(cmd.to_string(), TimeControl::new()))
        } else {
            CommIn::Unknown(cmd.to_string())
        }
    }
}

// ---------------------------------------------------------------------
// XBOARD output thread
// ---------------------------------------------------------------------

impl XBoard {
    // The control thread receives commands from the engine thread.
    fn output_thread(&mut self, board: Arc<Mutex<Board>>, options: Arc<Vec<EngineOption>>) {
        // Create an incoming channel for the control thread.
        let (output_tx, output_rx) = crossbeam_channel::unbounded::<CommOut>();

        // Create the output thread.
        let output_handle = thread::spawn(move || {
            let mut quit = false;
            let _t_options = Arc::clone(&options);

            // In the XBoard-protocol, the engine does not output stats all
            // the time like it does in UCI. It sends the stats when the
            // "." command comes in from the GUI. Therefore the output
            // thread will buffer the received stats in this struct,
            // instead of outputting them directly. This way the latest
            // stats are ready to send as soon as the GUI asks for them.
            let mut buf_stat01 = Stat01::new();

            // Keep running as long as Quit is not received.
            while !quit {
                let output = output_rx.recv().expect(ErrFatal::CHANNEL);

                // Perform command as sent by the engine thread.
                match output {
                    // Specific XBoard outputs
                    CommOut::XBoard(XBoardOut::NewLine) => XBoard::new_line(),
                    CommOut::XBoard(XBoardOut::Features) => XBoard::features(),
                    CommOut::XBoard(XBoardOut::Stat01) => XBoard::stat01(&buf_stat01),
                    CommOut::XBoard(XBoardOut::Pong(v)) => XBoard::pong(v),
                    CommOut::XBoard(XBoardOut::IllegalMove(m)) => XBoard::illegal_move(&m),

                    // Common outputs available to all protocols
                    CommOut::BestMove(m) => XBoard::best_move(&m),
                    CommOut::Result(score, reason) => XBoard::result(score, reason),
                    CommOut::OfferDraw => XBoard::offer_draw(),
                    CommOut::SearchSummary(summary) => {
                        XBoard::search_summary(&mut buf_stat01, &summary)
                    }
                    CommOut::SearchStats(stats) => XBoard::search_stats(&mut buf_stat01, &stats),
                    CommOut::SearchCurrMove(scm) => {
                        XBoard::search_current_move(&mut buf_stat01, &scm)
                    }
                    CommOut::Message(msg) => XBoard::message(&msg),
                    CommOut::Error(err_type, cmd) => XBoard::error(err_type, &cmd),
                    CommOut::Quit => quit = true,

                    // Custom prints for use in the console. These are
                    // available to all protocols.
                    CommOut::PrintBoard => Shared::print_board(&board),
                    CommOut::PrintHistory => Shared::print_history(&board),
                    CommOut::PrintEval(eval, phase) => Shared::print_eval(eval, phase),
                    CommOut::PrintState(state) => Shared::print_state(&state),
                    CommOut::PrintHelp => Shared::print_help(CommType::XBOARD),

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

impl XBoard {
    fn new_line() {
        println!();
    }

    fn features() {
        let myname = format!("myname=\"{} {}\"", About::ENGINE, About::VERSION);

        for f in FEATURES {
            let value = f.to_string().replace("myname=x", myname.as_str());
            println!("feature {}", value);
        }
    }

    fn pong(value: i8) {
        println!("pong {}", value)
    }

    fn message(msg: &str) {
        println!("# {}", msg);
    }

    fn illegal_move(m: &str) {
        println!("Illegal move: {}", m);
    }

    fn error(err_type: &str, cmd: &str) {
        println!("Error ({}): {}", err_type, cmd);
    }

    fn best_move(m: &Move) {
        println!("move {}", m);
    }

    fn search_summary(stat01: &mut Stat01, s: &SearchSummary) {
        // This function will cache the incoming search summary within the
        // "stat01" struct that lives in the output thread.
        stat01.depth = s.depth;
        stat01.time = s.time;
        stat01.nodes = s.nodes;

        // Report moves to checkmate if detected. Otherwise report normal score.
        let score = if let Some(moves) = Shared::moves_to_checkmate(s.cp) {
            // Flip the score if the engine is the one being mated.
            let flip = if s.cp < 0 { -1 } else { 1 };
            (CHECKMATE_VALUE + (moves as i32)) * flip
        } else {
            s.cp as i32
        };

        // After caching it will output the current search summary.
        // DEPTH SCORE TIME NODES PV
        println!(
            "{} {} {} {} {}",
            s.depth,
            score,
            (s.time as f64 / 10.0).round(),
            s.nodes,
            s.pv_to_string()
        );
    }

    fn search_stats(stat01: &mut Stat01, s: &SearchStats) {
        // Update the cached search stats with new time information.
        stat01.time = s.time;
        stat01.nodes = s.nodes;
    }

    fn search_current_move(stat01: &mut Stat01, scm: &SearchCurrentMove) {
        // Update cached search stats with new move information.s
        stat01.curr_move = scm.curr_move;
        stat01.curr_move_number = scm.curr_move_number;
        stat01.legal_moves_total = scm.legal_moves_total;
    }

    fn stat01(s: &Stat01) {
        // stat01: TIME NODES DEPTH MOVESLEFT TOTALMOVES CURRENTMOVE
        if s.curr_move.get_move() != 0 {
            println!(
                "stat01: {} {} {} {} {} {}",
                (s.time as f64 / 10.0).round(),
                s.nodes,
                s.depth,
                s.legal_moves_total - s.curr_move_number,
                s.legal_moves_total,
                s.curr_move
            );
        }
    }

    fn result(score: GameResult, reason: GameOverReason) {
        println!("{} {{{}}}", score, reason);
    }

    fn offer_draw() {
        println!("offer draw");
    }
}
