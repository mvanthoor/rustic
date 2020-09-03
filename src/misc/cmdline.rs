use crate::defs::{ABOUT, AUTHOR, EMAIL, ENGINE, VERSION};
use clap::{App, Arg, ArgMatches};

pub fn get() -> ArgMatches<'static> {
    App::new(ENGINE)
        .version(VERSION)
        .author(&*format!("{} <{}>", AUTHOR, EMAIL))
        .about(ABOUT)
        .arg(
            Arg::with_name("experiment")
                .short("x")
                .long("experiment")
                .takes_value(true)
                .help("This argument is being tested."),
        )
        .get_matches()
}
