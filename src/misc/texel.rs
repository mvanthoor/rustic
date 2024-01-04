mod defs;

use defs::DataFileLoadResult;
use defs::DataPoint;
use defs::TunerMessages;
use std::path::PathBuf;

pub struct Tuner {
    data_file_name: PathBuf,
    data_point: Vec<DataPoint>,
    lowest_mean_squared_error: f32,
}

impl Tuner {
    pub fn new(data_file_name: PathBuf) -> Self {
        Tuner {
            data_file_name,
            data_point: vec![],
            lowest_mean_squared_error: 0.0,
        }
    }

    pub fn run(&mut self) {
        if self.data_file_load().is_err() {
            println!("{}.", TunerMessages::DATA_FILE_NOT_FOUND);
        }

        println!("{}.", TunerMessages::DATA_FILE_LOADED);
    }

    fn data_file_load(&mut self) -> DataFileLoadResult {
        if !self.data_file_name.exists() {
            return Err(());
        }

        Ok(())
    }
}
