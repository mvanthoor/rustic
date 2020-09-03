use crate::defs::{ABOUT, AUTHOR, EMAIL, ENGINE, VERSION};
use clap::{App, Arg, ArgMatches};

// Consts for command line options, flags and arguments

// FEN
const CMD_FEN_LONG: &str = "fen";
const CMD_FEN_SHORT: &str = "f";
const CMD_FEN_HELP: &str = "Set up the given position.";

// Perft
const CMD_PERFT_LONG: &str = "perft";
const CMD_PERFT_SHORT: &str = "p";
const CMD_PERFT_HELP: &str = "Run perft to the given depth.";

// Interface
const CMD_INTERFACE_LONG: &str = "interface";
const CMD_INTERFACE_SHORT: &str = "i";
const CMD_INTERFACE_HELP: &str = "Values: uci, console";

pub fn get() -> ArgMatches<'static> {
    App::new(ENGINE)
        .version(VERSION)
        .author(&*format!("{} <{}>", AUTHOR, EMAIL))
        .about(ABOUT)
        .arg(
            Arg::with_name(CMD_INTERFACE_LONG)
                .short(CMD_INTERFACE_SHORT)
                .long(CMD_INTERFACE_LONG)
                .takes_value(true)
                .help(CMD_INTERFACE_HELP),
        )
        .arg(
            Arg::with_name(CMD_PERFT_LONG)
                .short(CMD_PERFT_SHORT)
                .long(CMD_PERFT_LONG)
                .takes_value(true)
                .help(CMD_PERFT_HELP),
        )
        .arg(
            Arg::with_name(CMD_FEN_LONG)
                .short(CMD_FEN_SHORT)
                .long(CMD_FEN_LONG)
                .takes_value(true)
                .help(CMD_FEN_HELP),
        )
        .get_matches()
}
