mod data_file;
mod data_point;
pub mod defs;
mod display;
mod init_eval;
mod k_factor;
mod result_types;

use data_file::{Line, LineParseError};
use data_point::DataPoint;
use librustic::board::{Board, defs::fen_setup_fast};
use result_types::{DataFileLineParseResult, DataFileLoadResult, TunerLoadError, TunerLoadResult};
use std::{
    fs::File,
    path::PathBuf,
    time::Instant,
    {io::BufRead, io::BufReader},
};

pub struct Tuner {
    board: Board,
    data_file_name: PathBuf,
    data_points: Vec<DataPoint>,
    k_factor: f32,
    min_mean_squared_error: f32,
}

impl Tuner {
    pub fn new(data_file_name: PathBuf) -> Self {
        Tuner {
            board: Board::new(),
            data_file_name,
            data_points: vec![],
            k_factor: 0.0,
            min_mean_squared_error: 0.0,
        }
    }

    pub fn load(&mut self) -> TunerLoadResult {
        let now = Instant::now();
        let data_file_store = match self.data_file_load() {
            Ok(store) => store,
            Err(_) => return Err(TunerLoadError::DataFileReadError),
        };

        self.print_data_file_read_result(&data_file_store);

        let lines = data_file_store.get_successful();
        let data_point_store = self.convert_lines_to_data_points(lines);
        self.data_points = data_point_store.get_successful().to_vec();

        self.print_data_point_conversion_result(&data_point_store);

        let elapsed = now.elapsed().as_millis();
        println!("Time taken: {elapsed} ms");
        println!("Data file loaded successfully");

        Ok(())
    }

    pub fn run(&mut self) {
        println!("Running tuner");
        println!("K-factor: Calculating");
        self.k_factor = self.calculate_k_factor();
        println!("K-factor: {}", self.k_factor);
    }
}

// Private functions
impl Tuner {
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
        let mut data_file_store = data_file::Store::new();

        for (i, line_result) in reader.lines().enumerate() {
            let i = i + 1;
            let line = match line_result {
                Ok(line) => line,
                Err(_) => {
                    data_file_store.insert_failed(Line::new(i, String::from("")));
                    continue;
                }
            };

            data_file_store.insert_successful(Line::new(i, line));
        }

        Ok(data_file_store)
    }

    fn convert_lines_to_data_points(&mut self, lines: &[Line]) -> data_point::Store {
        let mut data_point_store = data_point::Store::new();

        for line in lines {
            match self.parse_line_to_data_point(line) {
                Ok(data_point) => data_point_store.insert_successful(data_point),
                Err(error) => data_point_store.insert_failed(format!("{} - {}", line, error)),
            };
        }

        data_point_store
    }

    fn parse_line_to_data_point(&mut self, line: &Line) -> DataFileLineParseResult {
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
            return Err(LineParseError::DataLine);
        }

        // Create working variables.
        let fen = parts[0].clone();
        let result = parts[1].clone();

        // Validate the FEN-string by setting it up on a board.
        if fen_setup_fast(&mut self.board, Some(fen.trim())).is_err() {
            return Err(LineParseError::FenString);
        };

        // Try to parse the game result into an f32.
        let result = match result.trim() {
            "1-0" => 1.0,
            "1/2-1/2" => 0.5,
            "0-1" => 0.0,
            _ => return Err(LineParseError::GameResult),
        };

        // No errors? Return the data point.
        Ok(DataPoint::new(*line.get_nr(), fen, result, 0.0))
    }
}
