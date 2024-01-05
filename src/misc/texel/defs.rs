pub struct TunerMessages;
impl TunerMessages {
    pub const DATA_FILE_LOADED: &'static str = "Data file loaded.";
    pub const DATA_FILE_NOT_FOUND: &'static str = "Data file doesn't exist.";
    pub const ERROR_CANT_READ_LINE: &'static str = "Cannot read line. Skipped.";
}
