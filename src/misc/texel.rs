pub mod defs;

use defs::DataFileLoadResult;
use defs::TunerMessages;
use defs::TunerRunResult;
use defs::{DataPoint, DataPointParseError, DataPointParseResult};
use std::fs::File;
use std::path::PathBuf;
use std::{io, io::BufRead, io::BufReader};

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

        for (i, line) in reader.lines().enumerate() {
            let i = i + 1;
            if let Ok(line) = line {
                match self.parse_epd_line_to_datapoint(line) {
                    Ok(data_point) => {
                        self.data_point.push(data_point);
                        println!("{i} - OK");
                    }
                    Err(e) => match e {
                        DataPointParseError::ErrorInFenString => {
                            println!("{i} - {}", TunerMessages::ERROR_IN_FEN_STRING)
                        }
                        DataPointParseError::ErrorInGameResult => {
                            println!("{i} - {}", TunerMessages::ERROR_IN_GAME_RESULT)
                        }
                    },
                }
            } else {
                println!("{i} - {}", TunerMessages::ERROR_CANT_READ_LINE);
            }
        }

        Ok(())
    }

    fn parse_epd_line_to_datapoint(&mut self, line: String) -> DataPointParseResult {
        Ok(DataPoint::new(String::from(""), 0.0, 0.0))
    }
}
