use clap::{value_parser, Arg, ArgMatches};

// Consts for command line options, flags and arguments

struct CmdLineArgs {}
impl CmdLineArgs {
    const HASH_LONG: &'static str = "memory";
    const HASH_SHORT: char = 'm';
    const HASH_HELP: &'static str = "Transposition Table size in MB";
    const HASH_DEFAULT: usize = 32;
    const HASH_VALUE_NAME: &'static str = "megabytes";
}

pub struct CmdLine {
    arguments: ArgMatches,
}

impl Default for CmdLine {
    fn default() -> Self {
        Self::new()
    }
}

impl CmdLine {
    pub fn new() -> Self {
        Self {
            arguments: Self::get(),
        }
    }

    pub fn hash(&self) -> usize {
        *self
            .arguments
            .get_one::<usize>(CmdLineArgs::HASH_LONG)
            .unwrap_or(&CmdLineArgs::HASH_DEFAULT)
    }

    fn get() -> ArgMatches {
        let cmd_line = clap::Command::new("").version("").author("").about("").arg(
            Arg::new(CmdLineArgs::HASH_LONG)
                .short(CmdLineArgs::HASH_SHORT)
                .long(CmdLineArgs::HASH_LONG)
                .help(CmdLineArgs::HASH_HELP)
                .value_name(CmdLineArgs::HASH_VALUE_NAME)
                .value_parser(value_parser!(usize))
                .num_args(1),
        );

        cmd_line.get_matches()
    }
}
