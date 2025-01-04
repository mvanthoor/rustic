pub use crate::texel::result_types::TunerLoadError;
use std::path::PathBuf;

pub struct TexelSettings {
    pub file_name: Option<PathBuf>,
}

impl TexelSettings {
    pub fn new() -> Self {
        Self { file_name: None }
    }
}
