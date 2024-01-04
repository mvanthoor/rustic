use std::{
    fmt::{self, Display},
    io,
};

pub struct DataPoint {
    fen: String,
    result: f32,
    error: f32,
}

impl DataPoint {
    pub fn new(fen: String, result: f32, error: f32) -> Self {
        DataPoint { fen, result, error }
    }
}

pub enum DataPointParseError {
    ErrorInFenString,
    ErrorInGameResult,
}

impl Display for DataPointParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = match self {
            Self::ErrorInFenString => "Error in FEN string. Skipped.",
            Self::ErrorInGameResult => "Error in game result. Skipped.",
        };

        write!(f, "{m}")
    }
}

pub type DataFileLoadResult = io::Result<()>;
pub type DataPointParseResult = Result<DataPoint, DataPointParseError>;
pub type TunerRunResult = Result<(), ()>;

pub struct TunerMessages;
impl TunerMessages {
    pub const DATA_FILE_LOADED: &'static str = "Data file loaded.";
    pub const DATA_FILE_NOT_FOUND: &'static str = "Data file doesn't exist.";
    pub const ERROR_CANT_READ_LINE: &'static str = "Cannot read line. Skipped.";
}
