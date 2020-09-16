use crate::defs::{About, FEN_START_POSITION};
use clap::{App, Arg, ArgMatches};

// Consts for command line options, flags and arguments

struct CmdLineArgs {}
impl CmdLineArgs {
    // FEN
    const FEN_LONG: &'static str = "fen";
    const FEN_SHORT: &'static str = "f";
    const FEN_HELP: &'static str = "Set up the given position";

    // Perft
    const PERFT_LONG: &'static str = "perft";
    const PERFT_SHORT: &'static str = "p";
    const PERFT_HELP: &'static str = "Run perft to the given depth";
    const PERFT_DEFAULT: &'static str = "0";

    // Interface
    const COMM_LONG: &'static str = "comm";
    const COMM_SHORT: &'static str = "c";
    const COMM_HELP: &'static str = "Define communication to use";
    const COMM_VALUES: [&'static str; 3] = ["uci", "xboard", "console"];
    const COMM_DEFAULT: &'static str = "console";

    // Wizardry
    const WIZARDRY_LONG: &'static str = "wizardry";
    const WIZARDRY_SHORT: &'static str = "w";
    const WIZARDRY_HELP: &'static str = "Generate magic numbers";

    // Test
    const EPD_TEST_LONG: &'static str = "epdtest";
    const EPD_TEST_SHORT: &'static str = "e";
    const EPD_TEST_HELP: &'static str = "Run EPD Test Suite";
}

pub struct CmdLine {
    arguments: ArgMatches<'static>,
}

impl CmdLine {
    pub fn new() -> Self {
        Self {
            arguments: Self::get(),
        }
    }

    pub fn comm(&self) -> String {
        self.arguments
            .value_of(CmdLineArgs::COMM_LONG)
            .unwrap_or(CmdLineArgs::COMM_DEFAULT)
            .to_string()
    }

    pub fn fen(&self) -> String {
        self.arguments
            .value_of(CmdLineArgs::FEN_LONG)
            .unwrap_or(FEN_START_POSITION)
            .to_string()
    }

    pub fn perft(&self) -> u8 {
        self.arguments
            .value_of(CmdLineArgs::PERFT_LONG)
            .unwrap_or(CmdLineArgs::PERFT_DEFAULT)
            .parse()
            .unwrap_or(0)
    }

    #[cfg(feature = "extra")]
    pub fn wizardry(&self) -> bool {
        self.arguments.is_present(CmdLineArgs::WIZARDRY_LONG)
    }

    #[cfg(feature = "extra")]
    pub fn test(&self) -> bool {
        self.arguments.is_present(CmdLineArgs::EPD_TEST_LONG)
    }

    // &*format!("{} <{}>", About::AUTHOR, About::EMAIL)

    fn get() -> ArgMatches<'static> {
        let mut app = App::new(About::ENGINE)
            .version(About::VERSION)
            .author("Author X")
            .about(About::DESCRIPTION)
            .arg(
                Arg::with_name(CmdLineArgs::COMM_LONG)
                    .short(CmdLineArgs::COMM_SHORT)
                    .long(CmdLineArgs::COMM_LONG)
                    .help(CmdLineArgs::COMM_HELP)
                    .takes_value(true)
                    .default_value(CmdLineArgs::COMM_DEFAULT)
                    .possible_values(&CmdLineArgs::COMM_VALUES),
            )
            .arg(
                Arg::with_name(CmdLineArgs::FEN_LONG)
                    .short(CmdLineArgs::FEN_SHORT)
                    .long(CmdLineArgs::FEN_LONG)
                    .help(CmdLineArgs::FEN_HELP)
                    .takes_value(true)
                    .default_value(FEN_START_POSITION),
            )
            .arg(
                Arg::with_name(CmdLineArgs::PERFT_LONG)
                    .short(CmdLineArgs::PERFT_SHORT)
                    .long(CmdLineArgs::PERFT_LONG)
                    .help(CmdLineArgs::PERFT_HELP)
                    .takes_value(true)
                    .default_value(CmdLineArgs::PERFT_DEFAULT),
            );

        if cfg!(feature = "extra") {
            app = app
                .arg(
                    Arg::with_name(CmdLineArgs::WIZARDRY_LONG)
                        .short(CmdLineArgs::WIZARDRY_SHORT)
                        .long(CmdLineArgs::WIZARDRY_LONG)
                        .help(CmdLineArgs::WIZARDRY_HELP)
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name(CmdLineArgs::EPD_TEST_LONG)
                        .short(CmdLineArgs::EPD_TEST_SHORT)
                        .long(CmdLineArgs::EPD_TEST_LONG)
                        .help(CmdLineArgs::EPD_TEST_HELP)
                        .takes_value(false),
                );
        }

        app.get_matches()
    }
}
