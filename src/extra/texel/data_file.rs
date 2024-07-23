use std::fmt::{self, Display};

pub enum LineParseError {
    DataLine,
    FenString,
    GameResult,
}

impl Display for LineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = match self {
            Self::DataLine => "Error splitting line.",
            Self::FenString => "Error in FEN string.",
            Self::GameResult => "Error in game result.",
        };

        write!(f, "{m}")
    }
}

// Represents one line from the Texel tuning data file.
pub struct Line {
    nr: usize,
    line: String,
}

impl Line {
    pub fn new(nr: usize, line: String) -> Self {
        Line { nr, line }
    }

    pub fn get_nr(&self) -> &usize {
        &self.nr
    }

    pub fn get_line(&self) -> &str {
        &self.line
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line = format!("{} - {}", self.nr, self.line);
        write!(f, "{line}")
    }
}

// This struct holds the lines that where successfully read and which
// failed to read from the Texel data file.
pub struct Store {
    successful: Vec<Line>,
    failed: Vec<Line>,
}

impl Store {
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

    pub fn insert_successful_line(&mut self, line: Line) {
        self.successful.push(line);
    }

    pub fn insert_failed_line(&mut self, line: Line) {
        self.failed.push(line);
    }

    pub fn get_successful_lines(&self) -> &Vec<Line> {
        &self.successful
    }

    pub fn get_failed_lines(&self) -> &Vec<Line> {
        &self.failed
    }
}
