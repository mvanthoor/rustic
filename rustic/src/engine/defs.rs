use librustic::search::defs::Verbosity;

pub struct Settings {
    pub threads: usize,
    pub verbosity: Verbosity,
    pub tt_size: usize,
}
