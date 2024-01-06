use std::fmt::{self, Display};

pub enum DataFileLineParseError {
    ErrorInFenString,
    ErrorInGameResult,
}

impl Display for DataFileLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = match self {
            Self::ErrorInFenString => "Error in FEN string. Skipped.",
            Self::ErrorInGameResult => "Error in game result. Skipped.",
        };

        write!(f, "{m}")
    }
}

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
pub struct DataFileStore {
    successful: Vec<DataFileLine>,
    failed: Vec<DataFileLine>,
}

impl DataFileStore {
    pub fn new() -> Self {
        Self {
            successful: vec![],
            failed: vec![],
        }
    }

    pub fn count_successful_lines(&self) -> usize {
        self.successful.len()
    }

    pub fn count_failed_lines(&self) -> usize {
        self.failed.len()
    }

    pub fn count_all_lines(&self) -> usize {
        self.successful.len() + self.failed.len()
    }

    pub fn insert_successful_line(&mut self, line: DataFileLine) {
        self.successful.push(line);
    }

    pub fn insert_failed_line(&mut self, line: DataFileLine) {
        self.failed.push(line);
    }

    pub fn get_successful_lines(&self) -> &Vec<DataFileLine> {
        &self.successful
    }

    pub fn get_failed_lines(&self) -> &Vec<DataFileLine> {
        &self.failed
    }
}
