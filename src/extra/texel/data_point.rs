pub struct DataPoint {
    fen: String,
    result: f32,
    error: f32,
}

impl DataPoint {
    pub fn new(fen: String, result: f32, error: f32) -> Self {
        DataPoint { fen, result, error }
    }
}
