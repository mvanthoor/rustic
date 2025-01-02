use crate::search::defs::Verbosity;
use crate::{comm::defs::CommIn, search::defs::SearchReport};
use std::fmt::{Display, Formatter, Result};

#[cfg(feature = "extra")]
use std::path::PathBuf;

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

pub struct Messages;
impl Messages {
    pub const COMMAND_IGNORED: &'static str = "Command is known but unused";
    pub const INCOMING_CMD_BUFFERED: &'static str = "Incoming command buffered";
    pub const CLEARED_TT: &'static str = "Cleared the transposition table";
    pub const GAME_OVER: &'static str = "Game over. Result received";
}

// This struct holds the engine's settings.
#[cfg(feature = "extra")]
pub struct TexelSettings {
    pub file_name: Option<PathBuf>,
}
pub struct Settings {
    pub threads: usize,
    pub verbosity: Verbosity,
    pub tt_size: usize,

    #[cfg(feature = "extra")]
    pub texel: TexelSettings,
}

// This enum provides information to the engine, with regard to incoming
// messages and search results.
#[derive(PartialEq, Eq)]
pub enum Information {
    Comm(CommIn),
    Search(SearchReport),
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
