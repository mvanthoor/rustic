use crate::defs::About;
use clap::{App, Arg, ArgMatches};

// Consts for command line options, flags and arguments

// FEN
const CMD_FEN_LONG: &str = "fen";
const CMD_FEN_SHORT: &str = "f";
const CMD_FEN_HELP: &str = "Set up the given position";

// Perft
const CMD_PERFT_LONG: &str = "perft";
const CMD_PERFT_SHORT: &str = "p";
const CMD_PERFT_HELP: &str = "Run perft to the given depth";

// Interface
const CMD_COMM_LONG: &str = "comm";
const CMD_COMM_SHORT: &str = "c";
const CMD_COMM_HELP: &str = "Define communication to use";
const CMD_COMM_VALUES: [&str; 3] = ["uci", "xboard", "console"];

// Wizardry
const CMD_WIZARDRY_LONG: &str = "wizardry";
const CMD_WIZARDRY_SHORT: &str = "w";
const CMD_WIZARDRY_HELP: &str = "Generate magic numbers";

pub fn get() -> ArgMatches<'static> {
    App::new(About::ENGINE)
        .version(About::VERSION)
        .author(&*format!("{} <{}>", About::AUTHOR, About::EMAIL))
        .about(About::DESCRIPTION)
        .arg(
            Arg::with_name(CMD_COMM_LONG)
                .short(CMD_COMM_SHORT)
                .long(CMD_COMM_LONG)
                .help(CMD_COMM_HELP)
                .takes_value(true)
                .possible_values(&CMD_COMM_VALUES),
        )
        .arg(
            Arg::with_name(CMD_FEN_LONG)
                .short(CMD_FEN_SHORT)
                .long(CMD_FEN_LONG)
                .help(CMD_FEN_HELP)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(CMD_PERFT_LONG)
                .short(CMD_PERFT_SHORT)
                .long(CMD_PERFT_LONG)
                .help(CMD_PERFT_HELP)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(CMD_WIZARDRY_LONG)
                .short(CMD_WIZARDRY_SHORT)
                .long(CMD_WIZARDRY_LONG)
                .help(CMD_WIZARDRY_HELP)
                .takes_value(false),
        )
        .get_matches()
}
