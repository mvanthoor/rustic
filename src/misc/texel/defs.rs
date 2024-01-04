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

pub type DataFileLoadResult = Result<(), ()>;
pub type DataPointParseResult = Result<(), DataPointParseError>;

pub struct TunerMessages;
impl TunerMessages {
    pub const DATA_FILE_LOADED: &'static str = "Data file loaded";
    pub const DATA_FILE_NOT_FOUND: &'static str = "Data file doesn't exist";
}
