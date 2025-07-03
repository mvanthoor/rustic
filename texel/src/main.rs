/*
    // Texel tuner
    const TEXEL_LONG: &'static str = "texel";
    const TEXEL_VALUE_NAME: &'static str = "dataset.pgn";
    const TEXEL_HELP: &'static str = "Create new evaluation weights by Texel Tuning";

    pub fn texel(&self) -> Option<PathBuf> {
        let value = self.arguments.get_one::<PathBuf>(CmdLineArgs::TEXEL_LONG);
        if value.is_some() {
            value.cloned()
        } else {
            None
        }
    }

    .arg(
        Arg::new(CmdLineArgs::TEXEL_LONG)
            .long(CmdLineArgs::TEXEL_LONG)
            .help(CmdLineArgs::TEXEL_HELP)
            .value_name(CmdLineArgs::TEXEL_VALUE_NAME)
            .value_parser(value_parser!(PathBuf))
            .num_args(1),
    );
*/
mod texel;

use texel::{
    defs::{TexelSettings, TunerLoadError},
    Tuner,
};

fn main() -> Result<(), TunerLoadError> {
    let settings = TexelSettings::new();
    if let Some(data_file) = settings.file_name.to_owned() {
        let mut tuner = Tuner::new(data_file);

        match tuner.load() {
            Ok(()) => tuner.run(),
            Err(e) => match e {
                TunerLoadError::DataFileReadError => println!("{e}"),
            },
        };
    }

    Ok(())
}
