use crate::extra::texel::data_point::DataPoint;
use std::{
    fmt::{self, Display},
    io,
};

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
