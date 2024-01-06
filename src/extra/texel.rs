mod data_file;
mod data_point;
pub mod defs;
mod display;
mod result_types;

use crate::board::Board;
use data_file::{DataFileLine, DataFileLineParseError, DataFileStore};
use data_point::{DataPoint, DataPointStore};
use result_types::{DataFileLineParseResult, DataFileLoadResult, TunerRunError, TunerRunResult};
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use std::{io::BufRead, io::BufReader};

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
        let now = Instant::now();
        let data_file_store = match self.data_file_load() {
            Ok(store) => store,
            Err(_) => return Err(TunerRunError::DataFileReadError),
        };

        self.print_data_file_read_result(&data_file_store);

        let lines = data_file_store.get_successful_lines();
        let data_point_store = self.convert_lines_to_data_points(lines);
        self.data_points = data_point_store.get_successful_data_points().clone();

        self.print_data_point_conversion_result(&data_point_store);

        let elapsed = now.elapsed().as_secs();
        println!("Time taken: {elapsed} seconds");

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
        let mut data_file_store = DataFileStore::new();

        for (i, line_result) in reader.lines().enumerate() {
            let i = i + 1;
            let line = match line_result {
                Ok(line) => line,
                Err(_) => {
                    data_file_store.insert_failed_line(DataFileLine::new(i, String::from("")));
                    continue;
                }
            };

            data_file_store.insert_successful_line(DataFileLine::new(i, line));
        }

        Ok(data_file_store)
    }

    fn convert_lines_to_data_points(&self, lines: &Vec<DataFileLine>) -> DataPointStore {
        let mut data_point_store = DataPointStore::new();

        for line in lines {
            match self.parse_line_to_data_point(line) {
                Ok(data_point) => data_point_store.insert_successful(data_point),
                Err(error) => data_point_store.insert_failed_data(format!("{} - {}", line, error)),
            };
        }

        data_point_store
    }

    fn parse_line_to_data_point(&self, line: &DataFileLine) -> DataFileLineParseResult {
        const DASH: char = '-';
        const EM_DASH: char = '–';
        const SEMICOLON: char = ';';

        // Split the incoming line into multiple parts.
        let parts: Vec<String> = line
            .get_line()
            .replace(EM_DASH, DASH.encode_utf8(&mut [0; 4]))
            .split(SEMICOLON)
            .map(String::from)
            .collect();

        // It should have exactly two parts. If not, something is wrong
        // with the data formatting.
        if parts.len() != 2 {
            return Err(DataFileLineParseError::DataLine);
        }

        // Create working variables.
        let fen = parts[0].clone();
        let result = parts[1].clone();
        let mut board = Board::new();

        // Validate the FEN-string by setting it up on a board.
        if board.read_fen(Some(fen.trim())).is_err() {
            return Err(DataFileLineParseError::FenString);
        };

        // Try to parse the game result into an f32.
        let result = match result.trim() {
            "1-0" => 1.0,
            "1/2-1/2" => 0.5,
            "0-1" => 0.0,
            _ => return Err(DataFileLineParseError::GameResult),
        };

        // No errors? Return the data point.
        Ok(DataPoint::new(*line.get_nr(), fen, result, 0.0))
    }
}
