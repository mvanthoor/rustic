use crate::communication::{
    feature::{Feature, IFeature},
    xboard::defs::FEATURES,
};
use std::sync::Arc;

pub fn new_line() {
    println!();
}

pub fn features(engine: &str, version: &str, features: &Arc<Vec<Feature>>) {
    let myname = format!("myname=\"{engine} {version}\"");

    println!("feature done=0");

    for f in FEATURES {
        let value = f.to_string().replace("myname=x", myname.as_str());
        println!("feature {value}");
    }

    for f in features.iter() {
        let name = f.get_name();
        let option = format!("\"{name}\"");
        println!("feature option={option}");
    }

    println!("feature done=1");
}

pub fn error(error: String, cmd: String) {
    println!("error ({error}): {cmd}");
}

pub fn custom(info: String) {
    println!("{info}");
}
