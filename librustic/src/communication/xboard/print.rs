use crate::communication::xboard::defs::FEATURES;

pub fn new_line() {
    println!();
}

pub fn xboard_features(engine: &str, version: &str) {
    let myname = format!("myname=\"{} {}\"", engine, version);

    for f in FEATURES {
        let value = f.to_string().replace("myname=x", myname.as_str());
        println!("feature {value}");
    }
}

pub fn error(error: String, cmd: String) {
    println!("error ({error}): {cmd}");
}

pub fn custom(info: String) {
    println!("{info}");
}
