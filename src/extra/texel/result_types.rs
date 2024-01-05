use crate::extra::texel::data_file::DataFileInfo;
use crate::extra::texel::data_point::{DataPoint, DataPointParseError};

pub enum TunerRunError {
    DataFileReadError,
}

pub type DataFileLoadResult = Result<DataFileInfo, ()>;
pub type DataPointParseResult = Result<DataPoint, DataPointParseError>;
pub type TunerRunResult = Result<(), TunerRunError>;
