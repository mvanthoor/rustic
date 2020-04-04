pub mod bits;
pub mod parse;

use crate::defs::{AUTHOR, ENGINE, VERSION};

/** Prints information about the engine to the screen. */
pub fn engine_info() {
    println!();
    println!("Engine: {} {}", ENGINE, VERSION);
    println!("Author: {}", AUTHOR);
}
