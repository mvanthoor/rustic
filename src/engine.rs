use crate::defs::{AUTHOR, ENGINE, VERSION};
pub struct Engine;
impl Engine {
    pub fn new() -> Self {
        Self
    }

    pub fn about(&self) {
        println!();
        println!("Engine: {} {}", ENGINE, VERSION);
        println!("Author: {}", AUTHOR);
    }
}
