use crate::communication::{
    feature::{Feature, IFeature, UiElement},
    xboard::defs::XBOARD_FEATURES,
};
use std::sync::Arc;

pub fn new_line() {
    println!();
}

pub fn features(engine: &str, version: &str, features: &Arc<Vec<Feature>>) {
    let myname = format!("myname=\"{engine} {version}\"");

    println!("feature done=0");

    for f in XBOARD_FEATURES {
        let value = f.to_string().replace("myname=x", myname.as_str());
        println!("feature {value}");
    }

    for f in features.iter() {
        let prefix = String::from("feature option=");
        let name = f.get_name();
        let gui = f.get_ui_element();

        if let Some(element) = gui {
            match element {
                UiElement::Spin => {
                    let value = f.get_default().unwrap_or("");
                    let min = f.get_min().unwrap_or("");
                    let max = f.get_max().unwrap_or("");
                    let option = format!("{prefix}\"{name} -spin {value} {min} {max}\"");

                    println!("{option}");
                }
                UiElement::Button => {
                    let option = format!("{prefix}\"{name} -button\"");

                    println!("{option}");
                }
            }
        }
    }

    println!("feature done=1");
}

pub fn pong(n: isize) {
    println!("pong {n}");
}

pub fn error(error: String, cmd: String) {
    println!("error ({error}): {cmd}");
}

pub fn custom(info: String) {
    println!("{info}");
}
