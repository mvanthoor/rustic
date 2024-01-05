use crate::extra::texel::data_point::{DataPoint, DataPointParseError};
use std::io;

pub type DataFileLoadResult = io::Result<()>;
pub type DataPointParseResult = Result<DataPoint, DataPointParseError>;
pub type TunerRunResult = Result<(), ()>;
