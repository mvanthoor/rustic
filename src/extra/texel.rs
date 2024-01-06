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

use self::data_file::{DataFileLine, DataFileLineParseError, DataFileStore};
use self::data_point::DataPointStore;
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
        let data_file_store = match self.data_file_load() {
            Ok(store) => store,
            Err(_) => return Err(TunerRunError::DataFileReadError),
        };

        self.print_data_file_read_result(&data_file_store);
        self.convert_lines_to_data_points(data_file_store.get_successful_lines());

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
        let mut store = DataFileStore::new();

        for (i, line_result) in reader.lines().enumerate() {
            let i = i + 1;
            let line = match line_result {
                Ok(line) => line,
                Err(_) => {
                    store.insert_failed_line(DataFileLine::new(i, String::from("")));
                    continue;
                }
            };

            store.insert_successful_line(DataFileLine::new(i, line));
        }

        Ok(store)
    }

    fn print_data_file_read_result(&self, data_file_store: &DataFileStore) {
        println!(
            "Results reading data file: {}",
            self.data_file_name
                .clone()
                .into_os_string()
                .into_string()
                .unwrap_or_default()
        );
        println!("Lines read: {}", data_file_store.count_all_lines());
        println!(
            "Lines successful: {}",
            data_file_store.count_successful_lines()
        );

        if data_file_store.count_failed_lines() > 0 {
            println!("Lines failed: {}", data_file_store.count_failed_lines());
            for line in data_file_store.get_failed_lines() {
                println!("\tLine number: {}", line.get_nr());
            }
        }
    }

    fn convert_lines_to_data_points(&self, lines: &[DataFileLine]) -> DataPointStore {
        let mut data_point_store = DataPointStore::new();

        for line in lines {
            match self.parse_line_to_data_point(line) {
                Ok(data_point) => data_point_store.add_successful_data_point(data_point),
                Err(error) => data_point_store.add_failed_data(format!("{} - {}", line, error)),
            };
        }

        data_point_store
    }

    fn parse_line_to_data_point(&self, line: &DataFileLine) -> DataFileLineParseResult {
        Ok(DataPoint::new(1, String::from(""), 0.0, 0.0))
    }
}
