use crate::{comm::defs::CommIn, search::defs::SearchReport};
use std::fmt::{Display, Formatter, Result};

pub use crate::engine::transposition::{HashFlag, IHashData, PerftData, SearchData, TT};

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

// Lists all possible game results.
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
            GameResultReason::Other(reason) => write!(f, "{}", reason),
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

// This struct holds messages that are reported on fatal engine errors.
// These should never happen; if they do the engine is in an unknown state,
// and it will panic without trying any recovery whatsoever.
pub struct ErrFatal;
impl ErrFatal {
    pub const CREATE_COMM: &'static str = "Comm creation failed.";
    pub const NEW_GAME: &'static str = "Setting up new game failed.";
    pub const LOCK: &'static str = "Lock failed.";
    pub const READ_IO: &'static str = "Reading I/O failed.";
    pub const HANDLE: &'static str = "Broken handle.";
    pub const THREAD: &'static str = "Thread has failed.";
    pub const CHANNEL: &'static str = "Broken channel.";
    pub const NO_INFO_RX: &'static str = "No incoming Info channel.";
    pub const GENERATED_ILLEGAL_MOVE: &'static str = "The engine generated an illegal move!";
}

pub struct ErrNormal;
impl ErrNormal {
    pub const NOT_LEGAL: &'static str = "This is not a legal move in this position";
    pub const NOT_INT: &'static str = "The value given was not an integer";
    pub const FEN_FAILED: &'static str = "Setting up FEN failed";
    pub const UNKNOWN_COMMAND: &'static str = "Unknown command";
    pub const COMMAND_INVALID: &'static str = "Command invalid in current engine state";
    pub const INCORRECT_FEN: &'static str = "Incorrect FEN-string";
    pub const TIME_CONTROL_NOT_SET: &'static str = "Time control not set";
}

pub struct Messages;
impl Messages {
    pub const COMMAND_IGNORED: &'static str = "Command is known but unused";
    pub const INCOMING_CMD_BUFFERED: &'static str = "Incoming command buffered";
    pub const CLEARED_TT: &'static str = "Cleared the transposition table";
    pub const GAME_OVER: &'static str = "Game over. Result received";
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Verbosity {
    Full,
    Quiet,
    Silent,
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

// This struct holds the engine's settings.
pub struct Settings {
    pub threads: usize,
    pub verbosity: Verbosity,
    pub tt_size: usize,
}

// This enum provides information to the engine, with regard to incoming
// messages and search results.
#[derive(PartialEq, Eq)]
pub enum Information {
    Comm(CommIn),
    Search(SearchReport),
}

pub enum UiElement {
    Spin,
    Button,
}

pub struct EngineOption {
    pub name: &'static str,
    pub ui_element: UiElement,
    pub default: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
}

impl EngineOption {
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
            EngineSetOption::Hash(mb) => write!(f, "Hash {}", mb),
            EngineSetOption::ClearHash => write!(f, "Clear Hash"),
            EngineSetOption::Nothing => write!(f, ""),
        }
    }
}

pub struct EngineOptionDefaults;
impl EngineOptionDefaults {
    pub const HASH_DEFAULT: &'static str = "32";
    pub const HASH_MIN: &'static str = "0";
    pub const HASH_MAX_64_BIT: &'static str = "65536";
    pub const HASH_MAX_32_BIT: &'static str = "2048";
}
