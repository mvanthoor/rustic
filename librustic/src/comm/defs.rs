use crate::{
    board::Board,
    movegen::defs::Move,
    search::defs::{SearchCurrentMove, SearchReport, SearchStats, SearchSummary},
};
use std::{
    fmt::{Display, Formatter, Result},
    sync::{mpsc::Sender, Arc, Mutex},
};

pub use crate::comm::protocols::xboard::{TimeControl, XBoard, XBoardIn, XBoardOut};

#[derive(PartialEq, Eq, Clone)]
pub enum EngineSetOption {
    Hash(String),
    ClearHash,
    Nothing,
}

impl EngineSetOption {
    pub const HASH: &'static str = "Hash";
    pub const CLEAR_HASH: &'static str = "Clear Hash";
}

impl Display for EngineSetOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            EngineSetOption::Hash(mb) => write!(f, "Hash {mb}"),
            EngineSetOption::ClearHash => write!(f, "Clear Hash"),
            EngineSetOption::Nothing => write!(f, ""),
        }
    }
}

pub struct EngineOptionDefaults;
impl EngineOptionDefaults {
    pub const HASH_DEFAULT: usize = 32;
    pub const HASH_MIN: usize = 0;

    pub fn max_hash() -> usize {
        const HASH_MAX_64_BIT: usize = 65536;
        const HASH_MAX_32_BIT: usize = 2048;

        let is_64_bit = (std::mem::size_of::<usize>() * 8) == 64;

        if is_64_bit {
            HASH_MAX_64_BIT
        } else {
            HASH_MAX_32_BIT
        }
    }
}

pub struct Messages;
impl Messages {
    pub const COMMAND_IGNORED: &'static str = "Command is known but unused";
    pub const INCOMING_CMD_BUFFERED: &'static str = "Incoming command buffered";
    pub const CLEARED_TT: &'static str = "Cleared the transposition table";
    pub const GAME_OVER: &'static str = "Game over. Result received";
}

// This enum provides information to the engine, with regard to incoming
// messages and search results.
#[derive(PartialEq, Eq)]
pub enum Information {
    Comm(CommIn),
    Search(SearchReport),
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum EngineState {
    Observing,
    Waiting,
    Thinking,
    Analyzing,
}

impl Display for EngineState {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            EngineState::Observing => write!(f, "Observing"),
            EngineState::Waiting => write!(f, "Waiting"),
            EngineState::Thinking => write!(f, "Thinking"),
            EngineState::Analyzing => write!(f, "Analyzing"),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum GameResultPoints {
    WhiteWins,
    BlackWins,
    Draw,
    Asterisk,
    Nothing,
}

impl Display for GameResultPoints {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            GameResultPoints::WhiteWins => write!(f, "1-0"),
            GameResultPoints::BlackWins => write!(f, "0-1"),
            GameResultPoints::Draw => write!(f, "1/2-1/2"),
            GameResultPoints::Asterisk => write!(f, "*"),
            GameResultPoints::Nothing => write!(f, "-"),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum GameResultReason {
    WhiteMates,
    BlackMates,
    Stalemate,
    Insufficient,
    FiftyMoves,
    ThreeFold,
    Other(String),
    Nothing,
}

impl Display for GameResultReason {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            GameResultReason::WhiteMates => write!(f, "White mates"),
            GameResultReason::BlackMates => write!(f, "Black mates"),
            GameResultReason::Stalemate => write!(f, "Stalemate"),
            GameResultReason::ThreeFold => write!(f, "Draw by repetition"),
            GameResultReason::Insufficient => write!(f, "Insufficient material"),
            GameResultReason::FiftyMoves => write!(f, "Ffifty move rule"),
            GameResultReason::Other(reason) => write!(f, "{reason}"),
            GameResultReason::Nothing => write!(f, "-"),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct GameResult {
    pub points: GameResultPoints,
    pub reason: GameResultReason,
}

impl Display for GameResult {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} {{{}}}", self.points, self.reason)
    }
}

pub enum UiElement {
    Spin,
    Button,
}

pub struct CommOption {
    pub name: &'static str,
    pub ui_element: UiElement,
    pub default: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
}

impl CommOption {
    pub fn new(
        name: &'static str,
        ui_element: UiElement,
        default: Option<String>,
        min: Option<String>,
        max: Option<String>,
    ) -> Self {
        Self {
            name,
            ui_element,
            default,
            min,
            max,
        }
    }
}

// These are the types of communication the engine is capable of.
pub struct CommType;

impl CommType {
    pub const XBOARD: &'static str = "xboard";
    pub const UCI: &'static str = "uci";
}

pub struct CommInfo {
    protocol_name: &'static str,
    supports_fancy_about: bool,
    requires_stateful_mode: bool,
    requires_game_result: bool,
    startup_state: EngineState,
}

impl CommInfo {
    pub fn new(
        protocol_name: &'static str,
        supports_fancy_about: bool,
        requires_stateful_mode: bool,
        requires_game_result: bool,
        startup_state: EngineState,
    ) -> Self {
        Self {
            protocol_name,
            supports_fancy_about,
            requires_stateful_mode,
            requires_game_result,
            startup_state,
        }
    }

    pub fn protocol_name(&self) -> &str {
        self.protocol_name
    }

    pub fn supports_fancy_about(&self) -> bool {
        self.supports_fancy_about
    }

    pub fn requires_stateful_mode(&self) -> bool {
        self.requires_stateful_mode
    }

    pub fn requires_game_result(&self) -> bool {
        self.requires_game_result
    }

    pub fn startup_state(&self) -> EngineState {
        self.startup_state
    }
}

// Defines the public functions a Comm module must implement.
pub trait IComm {
    fn init(
        &mut self,
        report_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<CommOption>>,
    );
    fn send(&self, msg: CommOut);
    fn shutdown(&mut self);
    fn info(&self) -> &CommInfo;
}

#[derive(PartialEq, Eq, Clone)]
pub enum CommIn {
    XBoard(XBoardIn),

    // Common incoming commands
    Quit,
    Unknown(String),

    // Custom
    Board,
    History,
    Eval,
    State,
    Help,
    ClearTt,

    // Ignore an incoming command on purpose
    Ignore(String),
}

pub enum CommOut {
    XBoard(XBoardOut),

    // Common output for all protocols
    BestMove(Move, Option<GameResult>), // Transmit the engine's best move and result.
    SearchCurrMove(SearchCurrentMove),  // Transmit currently considered move.
    SearchSummary(SearchSummary),       // Transmit search information.
    SearchStats(SearchStats),           // Transmit search Statistics.
    Message(String),                    // Transmits a message to the GUI.
    Error(&'static str, String),        // Transmits an error message.
    Quit,                               // Terminates the output thread.

    // Output to screen when running in a terminal window.
    PrintBoard,
    PrintHistory,
    PrintEval(i16, i16),
    PrintState(EngineState),
    PrintHelp,
}
