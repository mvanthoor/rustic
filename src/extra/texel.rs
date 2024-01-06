mod data_file;
mod data_point;
pub mod defs;
mod result_types;

use data_file::DataFileLineParseError::{ErrorInFenString, ErrorInGameResult};
use data_point::DataPoint;
use result_types::{DataFileLineParseResult, DataFileLoadResult, TunerRunResult};
use std::fs::File;
use std::path::PathBuf;
use std::{io::BufRead, io::BufReader};

use self::data_file::{DataFileLine, DataFileStore};
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
        let store = match self.data_file_load() {
            Ok(store) => store,
            Err(_) => return Err(TunerRunError::DataFileReadError),
        };

        self.print_data_file_read_result(&store);

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
        let mut data_file = DataFileStore::new();

        for (i, line_result) in reader.lines().enumerate() {
            let i = i + 1;
            if line_result.is_err() || i == 3 || i == 7 {
                data_file.insert_failed_line(DataFileLine::new(i, String::from("")));
                continue;
            }

            let line = line_result.unwrap_or(String::from(""));
            data_file.insert_successful_line(DataFileLine::new(i, line));
        }

        Ok(data_file)
    }

    fn print_data_file_read_result(&self, store: &DataFileStore) {
        println!(
            "Results reading data file: {}",
            self.data_file_name
                .clone()
                .into_os_string()
                .into_string()
                .unwrap_or_default()
        );
        println!("Lines read: {}", store.count_all_lines());
        println!("Lines success: {}", store.count_successful_lines());
        println!("Lines failed: {}", store.count_failed_lines());

        for line in store.get_failed_lines() {
            println!("\tLine number: {}", line.get_nr());
        }
    }

    fn parse_epd_line_to_data_point(&mut self, line: String) -> DataFileLineParseResult {
        Ok(DataPoint::new(String::from(""), 0.0, 0.0))
    }
}
