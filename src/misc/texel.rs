mod defs;

use defs::DataPoint;
use std::path::PathBuf;

pub struct Tuner {
    data_file: PathBuf,
    data_point: Vec<DataPoint>,
    lowest_mean_squared_error: f32,
}

impl Tuner {
    pub fn new(data_file: PathBuf) -> Self {
        Tuner {
            data_file,
            data_point: vec![],
            lowest_mean_squared_error: 0.0,
        }
    }

    pub fn run(&self) {
        println!("Yes! Tuning! File exists: {}", self.data_file.exists());
    }
}
