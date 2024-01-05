use std::fmt::{self, Display};

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

pub struct DataPointInfo {
    success: Vec<DataPoint>,
    failed: Vec<String>,
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
