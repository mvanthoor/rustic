pub mod defs;

use defs::DataFileLoadResult;
use defs::DataPoint;
use defs::TunerMessages;
use std::path::PathBuf;

use self::defs::TunerRunResult;

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

    pub fn run(&mut self) -> TunerRunResult {
        if self.data_file_load().is_err() {
            println!("{}.", TunerMessages::DATA_FILE_NOT_FOUND);
            return Err(());
        }

        println!("{}.", TunerMessages::DATA_FILE_LOADED);
        Ok(())
    }

    fn data_file_load(&mut self) -> DataFileLoadResult {
        if !self.data_file_name.exists() {
            return Err(());
        }

        Ok(())
    }
}
