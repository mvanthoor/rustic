use librustic::search::defs::Verbosity;

#[cfg(feature = "extra")]
use std::path::PathBuf;

// This struct holds the engine's settings.
#[cfg(feature = "extra")]
pub struct TexelSettings {
    pub file_name: Option<PathBuf>,
}
pub struct Settings {
    pub threads: usize,
    pub verbosity: Verbosity,
    pub tt_size: usize,

    #[cfg(feature = "extra")]
    pub texel: TexelSettings,
}
