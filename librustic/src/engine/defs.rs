use crate::search::defs::Verbosity;
use std::fmt::{Display, Formatter, Result};

#[cfg(feature = "extra")]
use std::path::PathBuf;

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
