use crate::extra::texel::{
    data_file::{LineParseError, Store},
    data_point::DataPoint,
};
use std::fmt::{self, Display};

pub enum TunerLoadError {
    DataFileReadError,
}

impl Display for TunerLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = match self {
            Self::DataFileReadError => "Data file could not be read",
        };
        write!(f, "{error}")
    }
}

pub type DataFileLoadResult = Result<Store, ()>;
pub type DataFileLineParseResult = Result<DataPoint, LineParseError>;
pub type TunerLoadResult = Result<(), TunerLoadError>;
