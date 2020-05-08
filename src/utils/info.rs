use crate::defs::{AUTHOR, ENGINE, VERSION};

/** Prints information about the engine to the screen. */
pub fn about_engine() {
    println!();
    println!("Engine: {} {}", ENGINE, VERSION);
    println!("Author: {}", AUTHOR);
}
