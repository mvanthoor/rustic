mod data_file;
mod data_point;
pub mod defs;
mod result_types;

use data_point::DataPoint;
use data_point::DataPointParseError::{ErrorInFenString, ErrorInGameResult};
use result_types::{DataFileLoadResult, DataPointParseResult, TunerRunResult};
use std::fs::File;
use std::path::PathBuf;
use std::{io::BufRead, io::BufReader};

use self::data_file::{DataFile, DataFileLine};
use self::result_types::TunerRunError;

pub struct Tuner {
    data_file_name: PathBuf,
    data_points: Vec<DataPoint>,
    lowest_mean_squared_error: f32,
}

impl Tuner {
    pub fn new(data_file_name: PathBuf) -> Self {
        Tuner {
            data_file_name,
            data_points: vec![],
            lowest_mean_squared_error: 0.0,
        }
    }

    pub fn run(&mut self) -> TunerRunResult {
        let data_file = match self.data_file_load() {
            Ok(data_file) => data_file,
            Err(_) => return Err(TunerRunError::DataFileReadError),
        };

        self.print_data_file_read_result(&data_file);

        Ok(())
    }

    fn data_file_load(&mut self) -> DataFileLoadResult {
        if !self.data_file_name.exists() {
            return Err(());
        }

        let path = self.data_file_name.as_path();
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Err(()),
        };
        let reader = BufReader::new(file);
        let mut data_file = DataFile::new();

        for (i, line_result) in reader.lines().enumerate() {
            let i = i + 1;
            if line_result.is_err() {
                data_file.insert_failed(i);
                continue;
            }

            let line = line_result.unwrap_or(String::from(""));
            data_file.insert_success(DataFileLine::new(i, line));
        }

        Ok(data_file)
    }

    fn print_data_file_read_result(&self, data_file: &DataFile) {
        println!(
            "Results reading data file: {}",
            self.data_file_name
                .clone()
                .into_os_string()
                .into_string()
                .unwrap_or_default()
        );
        println!("Lines read: {}", data_file.count_all());
        println!("Lines success: {}", data_file.count_success());
        println!("Lines failed: {}", data_file.count_failed());

        for line in data_file.get_failed() {
            println!("\tLine number: {line}");
        }
    }

    fn parse_epd_line_to_data_point(&mut self, line: String) -> DataPointParseResult {
        Ok(DataPoint::new(String::from(""), 0.0, 0.0))
    }
}
