pub const ENGINE: &str = "Rustic";
pub const VERSION: &str = "Alpha 1";
pub const AUTHOR: &str = "Marcel Vanthoor";

pub struct Engine {}

impl Engine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn about(&self) {
        println!();
        println!("Engine: {} {}", ENGINE, VERSION);
        println!("Author: {}", AUTHOR);
    }
}
