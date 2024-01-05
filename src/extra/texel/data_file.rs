use std::fmt::{self, Display};

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
pub struct DataFileInfo {
    success: Vec<DataFileLine>,
    failed: Vec<usize>,
}

impl DataFileInfo {
    pub fn new() -> Self {
        Self {
            success: vec![],
            failed: vec![],
        }
    }

    pub fn count_success(&self) -> usize {
        self.success.len()
    }

    pub fn count_failed(&self) -> usize {
        self.failed.len()
    }

    pub fn count_all(&self) -> usize {
        self.success.len() + self.failed.len()
    }

    pub fn insert_success(&mut self, line: DataFileLine) {
        self.success.push(line);
    }

    pub fn insert_failed(&mut self, line_nr: usize) {
        self.failed.push(line_nr);
    }

    pub fn get_success(&self) -> &Vec<DataFileLine> {
        &self.success
    }

    pub fn get_failed(&self) -> &Vec<usize> {
        &self.failed
    }
}
