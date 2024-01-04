use std::io;

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

pub type DataFileLoadResult = io::Result<()>;
pub type DataPointParseResult = Result<DataPoint, DataPointParseError>;
pub type TunerRunResult = Result<(), ()>;

pub struct TunerMessages;
impl TunerMessages {
    pub const DATA_FILE_LOADED: &'static str = "Data file loaded.";
    pub const DATA_FILE_NOT_FOUND: &'static str = "Data file doesn't exist.";
    pub const ERROR_IN_FEN_STRING: &'static str = "Error in FEN string. Skipped.";
    pub const ERROR_IN_GAME_RESULT: &'static str = "Error in game result. Skipped.";
    pub const ERROR_CANT_READ_LINE: &'static str = "Cannot read line. Skipped.";
}
