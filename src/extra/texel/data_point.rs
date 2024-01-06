use std::fmt::{self, Display};

pub struct DataPoint {
    line_nr: usize,
    fen: String,
    result: f32,
    eval_error: f32,
}

impl DataPoint {
    pub fn new(line_nr: usize, fen: String, result: f32, eval_error: f32) -> Self {
        DataPoint {
            line_nr,
            fen,
            result,
            eval_error,
        }
    }
}

impl Display for DataPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line = format!(
            "Line: {} FEN: {} Result: {} EvalError: {}",
            self.line_nr, self.fen, self.result, self.eval_error
        );
        write!(f, "{line}")
    }
}

pub struct DataPointStore {
    successful: Vec<DataPoint>,
    failed: Vec<String>,
}

impl DataPointStore {
    pub fn new() -> Self {
        Self {
            successful: vec![],
            failed: vec![],
        }
    }

    pub fn count_all(&self) -> usize {
        self.successful.len() + self.failed.len()
    }

    pub fn count_successful_data_points(&self) -> usize {
        self.successful.len()
    }

    pub fn count_failed_data(&self) -> usize {
        self.failed.len()
    }

    pub fn insert_successful_data_point(&mut self, data_point: DataPoint) {
        self.successful.push(data_point);
    }

    pub fn insert_failed_data(&mut self, s: String) {
        self.failed.push(s);
    }
}
