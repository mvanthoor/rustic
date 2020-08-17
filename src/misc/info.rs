use crate::defs::{AUTHOR, ENGINE, VERSION};

pub fn about() {
    println!();
    println!("Engine: {} {}", ENGINE, VERSION);
    println!("Author: {}", AUTHOR);
}
