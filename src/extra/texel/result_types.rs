use crate::extra::texel::{
    data_file::DataFileLineParseError, data_file::DataFileStore, data_point::DataPoint,
};

pub enum TunerRunError {
    DataFileReadError,
}

pub type DataFileLoadResult = Result<DataFileStore, ()>;
pub type DataFileLineParseResult = Result<DataPoint, DataFileLineParseError>;
pub type TunerRunResult = Result<(), TunerRunError>;
