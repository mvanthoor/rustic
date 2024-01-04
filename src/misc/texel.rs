pub mod defs;

use defs::DataFileLoadResult;
use defs::TunerMessages;
use defs::TunerRunResult;
use defs::{
    DataPoint,
    DataPointParseError::{ErrorInFenString, ErrorInGameResult},
    DataPointParseResult,
};
use std::fs::File;
use std::path::PathBuf;
use std::{io, io::BufRead, io::BufReader};

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
        if self.data_file_load().is_err() {
            println!("{}", TunerMessages::DATA_FILE_NOT_FOUND);
            return Err(());
        }

        println!("{}", TunerMessages::DATA_FILE_LOADED);
        Ok(())
    }

    fn data_file_load(&mut self) -> DataFileLoadResult {
        if !self.data_file_name.exists() {
            return Err(io::Error::from(io::ErrorKind::NotFound));
        }

        let path = self.data_file_name.as_path();
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (i, line_result) in reader.lines().enumerate() {
            if line_result.is_err() {
                println!("{i} - {}", TunerMessages::ERROR_CANT_READ_LINE);
                continue;
            }

            let line = line_result.unwrap_or(String::from(""));
            let parse_result = self.parse_epd_line_to_data_point(line);
            let data_point = match parse_result {
                Ok(data_point) => data_point,
                Err(error) => {
                    match error {
                        ErrorInFenString => println!("{}", ErrorInFenString),
                        ErrorInGameResult => println!("{}", ErrorInGameResult),
                    }
                    continue;
                }
            };

            self.data_points.push(data_point);
        }

        Ok(())
    }

    fn parse_epd_line_to_data_point(&mut self, line: String) -> DataPointParseResult {
        Ok(DataPoint::new(String::from(""), 0.0, 0.0))
    }
}
