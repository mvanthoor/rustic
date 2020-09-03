use crate::defs::{ABOUT, AUTHOR, EMAIL, ENGINE, VERSION};

pub fn about() {
    println!();
    println!("Engine: {} {}", ENGINE, VERSION);
    println!("About: {}", ABOUT);
    println!("Author: {} <{}>", AUTHOR, EMAIL);
}
