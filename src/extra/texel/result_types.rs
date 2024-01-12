use crate::extra::texel::{
    data_file::DataFileLineParseError, data_file::DataFileStore, data_point::DataPoint,
};
use std::fmt::{self, Display};

pub enum TunerLoadrror {
    DataFileReadError,
}

impl Display for TunerLoadrror {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = match self {
            Self::DataFileReadError => "Data file could not be read",
        };
        write!(f, "{error}")
    }
}

pub type DataFileLoadResult = Result<DataFileStore, ()>;
pub type DataFileLineParseResult = Result<DataPoint, DataFileLineParseError>;
pub type TunerLoadResult = Result<(), TunerLoadrror>;
