use crate::defs::About;
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

    // Interface
    const COMM_LONG: &'static str = "comm";
    const COMM_SHORT: &'static str = "c";
    const COMM_HELP: &'static str = "Define communication to use";
    const COMM_VALUES: [&'static str; 3] = ["uci", "xboard", "console"];

    // Wizardry
    const WIZARDRY_LONG: &'static str = "wizardry";
    const WIZARDRY_SHORT: &'static str = "w";
    const WIZARDRY_HELP: &'static str = "Generate magic numbers";
}

pub fn get() -> ArgMatches<'static> {
    App::new(About::ENGINE)
        .version(About::VERSION)
        .author(&*format!("{} <{}>", About::AUTHOR, About::EMAIL))
        .about(About::DESCRIPTION)
        .arg(
            Arg::with_name(CmdLineArgs::COMM_LONG)
                .short(CmdLineArgs::COMM_SHORT)
                .long(CmdLineArgs::COMM_LONG)
                .help(CmdLineArgs::COMM_HELP)
                .takes_value(true)
                .possible_values(&CmdLineArgs::COMM_VALUES),
        )
        .arg(
            Arg::with_name(CmdLineArgs::FEN_LONG)
                .short(CmdLineArgs::FEN_SHORT)
                .long(CmdLineArgs::FEN_LONG)
                .help(CmdLineArgs::FEN_HELP)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(CmdLineArgs::PERFT_LONG)
                .short(CmdLineArgs::PERFT_SHORT)
                .long(CmdLineArgs::PERFT_LONG)
                .help(CmdLineArgs::PERFT_HELP)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(CmdLineArgs::WIZARDRY_LONG)
                .short(CmdLineArgs::WIZARDRY_SHORT)
                .long(CmdLineArgs::WIZARDRY_LONG)
                .help(CmdLineArgs::WIZARDRY_HELP)
                .takes_value(false),
        )
        .get_matches()
}
