use crate::extra::texel::data_point::DataPoint;
use crate::extra::texel::data_point::DataPointParseError;
use std::{
    fmt::{self, Display},
    io,
};

// Represents one line from the Texel tuning data file.
pub struct DataFileLine {
    nr: usize,
    line: String,
}

impl DataFileLine {
    pub fn new(nr: usize, line: String) -> Self {
        DataFileLine { nr, line }
    }

    pub fn get_nr(&self) -> &usize {
        &self.nr
    }

    pub fn get_line(&self) -> &str {
        &self.line
    }
}

impl Display for DataFileLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line = format!("{} - {}", self.nr, self.line);
        write!(f, "{line}")
    }
}

// This struct holds the lines that where successfully read and which
// failed to read from the Texel data file.
pub struct DataFileLineInfo {
    success: Vec<DataFileLine>,
    failed: Vec<DataFileLine>,
}

impl DataFileLineInfo {
    pub fn get_success(&self) -> &Vec<DataFileLine> {
        &self.success
    }

    pub fn get_failed(&self) -> &Vec<DataFileLine> {
        &self.failed
    }
}
