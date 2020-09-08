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
const CMD_INTERFACE_LONG: &str = "interface";
const CMD_INTERFACE_SHORT: &str = "i";
const CMD_INTERFACE_HELP: &str = "Values: uci, console";

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
            Arg::with_name(CMD_INTERFACE_LONG)
                .short(CMD_INTERFACE_SHORT)
                .long(CMD_INTERFACE_LONG)
                .help(CMD_INTERFACE_HELP)
                .takes_value(true),
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
