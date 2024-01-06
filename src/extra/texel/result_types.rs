use crate::extra::texel::data_file::DataFileStore;
use crate::extra::texel::data_point::{DataFileLineParseError, DataPoint};

pub enum TunerRunError {
    DataFileReadError,
}

pub type DataFileLoadResult = Result<DataFileStore, ()>;
pub type DataPointParseResult = Result<DataPoint, DataFileLineParseError>;
pub type TunerRunResult = Result<(), TunerRunError>;
